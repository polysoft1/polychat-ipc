use crate::{
    core::socket_handler::SocketHandler,
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
use tokio::{task::JoinHandle, sync::{ Mutex, mpsc::{self, Receiver, Sender}}, time::timeout};

#[derive(Debug)]
pub struct Process {
    child: Child,
    process_path: PathBuf,
    core_read_thread: JoinHandle<()>,
    socket: Arc<Mutex<SocketHandler>>,
    rx: Receiver<DeserializableCoreInstr>
}

impl Process {
    pub fn new<T>(path: T, socket: SocketHandler) -> Result<Process> where PathBuf: From<T> {
        let path = PathBuf::from(path);
        let (tx, rx) = mpsc::channel(100);
        let socket = Arc::new(Mutex::new(socket));
        let thrd_socket = socket.clone();

        match Command::new(&path).stdout(Stdio::null()).spawn() {
            Ok(child) => {
                debug!("Successfully started process {:?} with PID {}", path, child.id());
                Ok(Process {
                    child,
                    core_read_thread: tokio::spawn(async move {
                        fetch_message_loop(thrd_socket, tx).await;
                    }),
                    process_path: path,
                    rx,
                    socket
                })
            },
            Err(e) => {
                debug!("Could not load process from path {:?}: {}", path, e);
                Err(e.into())
            }
        }
    }

    pub async fn get_next_instruction(&mut self) -> Result<Option<DeserializableCoreInstr>> {
        match self.rx.recv().await {
            Some(v) => Ok(Some(v)),
            None => Ok(None)
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

async fn fetch_message_loop(socket: Arc<Mutex<SocketHandler>>, tx: Sender<DeserializableCoreInstr>) {
    let mut msg_buffer = Vec::new();
    loop {
        trace!("Attempting to aquire lock to SocketHandler");
        match timeout(Duration::from_millis(16), socket.lock()).await {
            Ok(mut lock) => {
                // Receive data from socket
                trace!("Getting data from SocketHandler");
                match timeout(Duration::from_millis(16), lock.get_instruction()).await {
                    Ok(v) => {
                        match v {
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
    use crate::{process_management::process::Process, core::socket_handler::SocketHandler};
    use claims::assert_ok;
    use test_log::test;

    #[cfg(target_os = "windows")]
    const TEST_PROGRAM: &str = "calc.exe";
    #[cfg(not(target_os = "windows"))]
    const TEST_PROGRAM: &str = "yes";

    #[test(tokio::test)]
    async fn test_loading_process() {
        assert_ok!(Process::new(TEST_PROGRAM, create_socket("polychat-loading-test")));
    }

    #[test(tokio::test)]
    async fn test_dropping_process() {
        let proc = assert_ok!(Process::new(TEST_PROGRAM, create_socket("polychat-drop-test")));

        drop(proc);
    }

    fn create_socket(name: &str) -> SocketHandler {
        let socket = SocketHandler::new(name);
        assert_ok!(socket, "Could not initialize SocketHandler")
    }
}