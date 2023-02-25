use crate::{core::socket_handler::SocketHandler, api::schema::instructions::{CoreInstruction, PluginInstruction}};

use std::{
    process::{Child, Command, Stdio},
    fmt::Debug, path::PathBuf,
    sync::{mpsc::{self, Receiver, Sender}, Arc}
};
use log::{warn, debug, error, trace};

use anyhow::Result;
use tokio::{task::JoinHandle, sync::Mutex};

#[derive(Debug)]
pub struct Process {
    child: Child,
    process_path: PathBuf,
    core_read_thread: JoinHandle<()>,
    socket: Arc<Mutex<SocketHandler>>,
    rx: Receiver<Result<CoreInstruction>>
}

impl Process {
    pub fn new<T>(path: T, socket: SocketHandler) -> Result<Process> where PathBuf: From<T> {
        let path = PathBuf::from(path);
        let (tx, rx) = mpsc::channel();
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

    /**
     * Returns if the next core instruction is available
     * 
     * # Returns
     * A [bool] on if the next instruction is available
     **/
    pub fn poll_next_instruction(&self) -> Option<Result<CoreInstruction>> {
        match self.rx.try_recv() {
            Ok(v) => Some(v),
            Err(e) => {
                match e {
                    mpsc::TryRecvError::Empty => None,
                    mpsc::TryRecvError::Disconnected => {
                        error!("Sending channel for {} disconnected, this should NEVER happen", self.process_path.display());
                        Some(Err(e.into()))
                    }
                }
            }
        }
    }

    pub fn get_next_instruction(&mut self) -> Result<CoreInstruction> {
        match self.rx.recv() {
            Ok(v) => v,
            Err(e) => Err(e.into())
        }
    }

    pub async fn send_instruction(&mut self, inst: &PluginInstruction) -> Result<()>{
        let mut lock = match self.socket.try_lock() {
            Err(e) => {
                debug!("Could not obtain lock: {}", e);
                return Err(e.into());
            }
            Ok(v) => v,
        };

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

async fn fetch_message_loop(socket: Arc<Mutex<SocketHandler>>, tx: Sender<Result<CoreInstruction>>) {
    loop {
        let mut lock = match socket.try_lock() {
            Ok(v) => v,
            Err(e) => {
                debug!("Could not get lock for socket: {}", e);
                continue;
            }
        };
        let data = lock.get_instruction().await;
        match &data {
            Ok(v) => {
                trace!("Sending result {}", v);
            }
            Err(e) => {
                trace!("Sending error {}", e);
            }
        };
        match tx.send(data) {
            Err(e) => {
                error!("Could not send instruction {}", e);
            }
            _ => {
                trace!("Send successful");
            }
        };
    }
}

#[cfg(test)]
mod test {
    use crate::{process_management::process::Process, core::socket_handler::SocketHandler};
    use tokio_test::assert_ok;
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