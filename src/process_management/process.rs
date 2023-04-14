use crate::{
    process_management::ipc_server::IPCServer,
    api::schema::instructions::{DeserializableCoreInstr, SerializablePluginInstr}
};

use std::{
    process::{Child, Command, Stdio},
    fmt::Debug, path::PathBuf,
    sync::Arc, time::Duration,
    thread::sleep
};
use log::{warn, debug, error, trace};

use anyhow::Result;
use serde::Serialize;
use tokio::{task::JoinHandle, sync::{ Mutex, mpsc::Sender}, time::timeout};

#[derive(Debug)]
pub struct Process {
    child: Child,
    process_path: PathBuf,
    core_read_thread: JoinHandle<()>,
    socket: Arc<Mutex<IPCServer>>,
}

impl Process {
    pub fn new<T>(path: T, socket: IPCServer, shared_queue_tx: Sender<DeserializableCoreInstr>) -> Result<Process> where PathBuf: From<T> {
        let socket_name_arg = socket.get_socket_name().clone();
        let path = PathBuf::from(path);
        let socket = Arc::new(Mutex::new(socket));
        let thrd_socket = socket.clone();
        debug!("Starting process at {:?} with socket name argument {}", &path, &socket_name_arg);

        match Command::new(&path).arg(socket_name_arg).stdout(Stdio::null()).spawn() {
            Ok(child) => {
                debug!("Successfully started process {:?} with PID {}", path, child.id());
                Ok(Process {
                    child,
                    core_read_thread: tokio::spawn(async move {
                        fetch_message_loop(thrd_socket, shared_queue_tx).await;
                    }),
                    process_path: path,
                    socket
                })
            },
            Err(e) => {
                debug!("Could not load process from path {:?}: {}", path, e);
                Err(e.into())
            }
        }
    }

    pub async fn send_instruction<P: Serialize + Debug>(&mut self, inst: &SerializablePluginInstr<P>) -> Result<()>{
        debug!("Awaiting lock to send data across tasks");
        let mut lock = self.socket.lock().await;
        debug!("Lock acquired to send data across tasks");
        lock.send_plugin_instruction(inst).await
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let id = self.child.id();
        self.core_read_thread.abort();
        // TODO: Handle the case where the process terminates early, either
        // by checking if it's still running, or gracefully failing on kill.
        // On Windows, if it terminates early, it's an access denied error.
        match self.child.try_wait() {
            Ok(Some(status)) => {
                debug!("Process {} exited early: code {}", self.process_path.display(), status);
                return;
            },
            Ok(None) => {}, // Process still running
            Err(e) => {
                error!("Error checking state of process {}, reason: {}", self.process_path.display(), e);
                return;
            }
        };

        match self.child.kill() {
            Err(e) => {
                warn!("Could not kill process {}: {}", id, e);
                #[cfg(test)]
                assert!(false, "Error killing process {}: {}", id, e);

                return;
            },
            Ok(()) => debug!("Successfully killed process {}", id)
        };

        match self.child.wait() {
            Err(e) => {
                warn!("Process {} did not exit: {}", id, e);
                #[cfg(test)]
                assert!(false, "Error closing process {}: {}", id, e);
            },
            Ok(code) => debug!("Process {} exited with code: {}", id, code)
        };
    }
}

async fn fetch_message_loop(socket: Arc<Mutex<IPCServer>>, tx: Sender<DeserializableCoreInstr>) {
    // Temporarily stores messages that were received by the core into a buffer, then sends them to the tx Sender.
    let mut msg_buffer = Vec::new();
    loop {
        trace!("Attempting to aquire lock to SocketHandler");
        match timeout(Duration::from_millis(16), socket.lock()).await {
            Ok(mut lock) => {
                // Receive data from socket. This is data from the plugin to the core.
                trace!("Getting data from SocketHandler");
                match timeout(Duration::from_millis(16), lock.get_instruction()).await {
                    Ok(v) => {
                        match v {
                            // Add the instruction to the buffer so it can be processed later.
                            Ok(d) => msg_buffer.push(d),
                            Err(e) => {
                                warn!("Error obtaining next core instruction: {}", e);
                            }
                        }
                    },
                    Err(_) => {
                        trace!("Receive timed out");
                    }
                };
            },
            Err(_) => {}
        }

        if msg_buffer.len() > 0 {
            let rx_data = msg_buffer.get(0).unwrap();

            // Sent to parent thread
            match timeout(Duration::from_millis(16), tx.send(rx_data.clone())).await {
                Err(e) => {
                    error!("Could not send instruction {}", e);
                }
                _ => {
                    trace!("Send successful");
                    msg_buffer.remove(0);
                }
            };   
        }
        // TODO: Determine if the loop efficiently waits, and if so, remove this.
        sleep(Duration::from_millis(16));
    }
}

#[cfg(test)]
mod test {
    use crate::process_management::{process::Process, ipc_server::IPCServer};
    use claims::assert_ok;
    use test_log::test;
    use tokio::sync::mpsc;

    #[cfg(target_os = "windows")]
    const TEST_PROGRAM: &str = "calc.exe";
    #[cfg(not(target_os = "windows"))]
    const TEST_PROGRAM: &str = "yes";

    #[test(tokio::test)]
    async fn test_loading_process() {
        let (tx, _rx) = mpsc::channel(100);
        assert_ok!(Process::new(TEST_PROGRAM, create_socket("polychat-loading-test"), tx));
    }

    #[test(tokio::test)]
    async fn test_dropping_process() {
        let (tx, _rx) = mpsc::channel(100);
        let proc = assert_ok!(Process::new(TEST_PROGRAM, create_socket("polychat-drop-test"), tx));

        drop(proc);
    }

    fn create_socket(name: &str) -> IPCServer {
        let socket = IPCServer::new(name);
        assert_ok!(socket, "Could not initialize SocketHandler")
    }
}
