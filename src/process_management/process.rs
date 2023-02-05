use std::{
    process::{Child, Command, Stdio},
    ffi::OsStr
};

use log::{warn, debug};

#[cfg(test)]
use test_log::test;

#[derive(Debug)]
pub struct Process {
    child: Child,
}

impl Process {
    pub fn new<T: AsRef<OsStr> + std::fmt::Debug>(path: T) -> Result<Process, String> {
        let child = Command::new(&path).stdout(Stdio::null()).spawn();

        match child {
            Ok(child) => {
                debug!("Successfully started process {:?} with PID {}", path, child.id());
                Ok(Process {
                    child: child
                })
            },
            Err(e) => {
                debug!("Could not load process from path {:?}: {}", &path, e);
                Err(
                    format!("Could not load process from path {:?}: {}", path, e)
                )
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
        match self.child.kill() {
            Err(e) => {
                warn!("Could not kill process {}: {}", id, e);
                #[cfg(test)]
                assert!(false, "Error killing process {}: {}", id, e);
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
#[cfg(target_os = "windows")]
const TEST_PROGRAM: &str = "calc.exe";
#[cfg(test)]
#[cfg(not(target_os = "windows"))]
const TEST_PROGRAM: &str = "yes";

#[test]
fn test_loading_process() {
    let proc = Process::new(TEST_PROGRAM);

    assert!(proc.is_ok(), "Could not load program \"{TEST_PROGRAM}\"");
}

#[test]
fn test_dropping_process() {
    let proc = Process::new(TEST_PROGRAM);

    assert!(proc.is_ok(), "Could not load program \"{TEST_PROGRAM}\"");

    drop(proc.unwrap());
}