use crate::core::socket_handler::SocketHandler;

use std::{
    process::{Child, Command, Stdio},
    fmt::Debug, path::PathBuf
};
use log::{warn, debug, error};

use anyhow::Result;

#[derive(Debug)]
pub struct Process {
    child: Child,
    socket: SocketHandler,
    process_path: PathBuf
}

impl Process {
    pub fn new<T>(path: T, socket: SocketHandler) -> Result<Process> where PathBuf: From<T> {
        let path = PathBuf::from(path);

        match Command::new(&path).stdout(Stdio::null()).spawn() {
            Ok(child) => {
                debug!("Successfully started process {:?} with PID {}", path, child.id());
                Ok(Process {
                    child: child,
                    socket,
                    process_path: path
                })
            },
            Err(e) => {
                debug!("Could not load process from path {:?}: {}", path, e);
                Err(e.into())
            }
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let id = self.child.id();
        // TODO: Handle the case where the process terminates early, either
        // by checking if it's still running, or gracefully failing on kill.
        // On Windows, if it terminates early, it's an access denied error.
        match self.child.try_wait() {
            Ok(Some(status)) => {
                debug!("Process exited early: code {}", status);
                return;
            },
            Ok(None) => {}, // Process still running
            Err(e) => {
                error!("Error checking state of process, reason: {}", e);
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