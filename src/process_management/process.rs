use std::process::{Child, Command};

use log::{warn, debug};

#[cfg(test)]
use test_log::test;

struct Process {
    child: Child,
}

impl Process {
    pub fn new(path: &str) -> Result<Process, String> {
        let child = Command::new(&path).spawn();

        match child {
            Ok(child) => {
                debug!("Successfully started process {} with PID {}", path, child.id());
                Ok(Process {
                    child: child
                })
            },
            Err(e) => {
                debug!("Could not load process from path {}: {}", &path, e);
                Err(
                    format!("Could not load process from path {}: {}", path, e)
                )
            }
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let id = self.child.id();
        match self.child.kill() {
            Err(e) => warn!("Could not kill process {}: {}", id, e),
            Ok(()) => debug!("Successfully killed process {}", id)
        };

        match self.child.wait() {
            Err(e) => warn!("Process {} did not exit: {}", id, e),
            Ok(code) => debug!("Process {} exited with code: {}", id, code)
        };
    }
}


#[test]
fn test_loading_process() {
    let proc = Process::new("yes");

    assert!(proc.is_ok(), "Could not load program \"yes\"");
}

#[test]
fn test_dropping_process() {
    let proc = Process::new("yes");

    assert!(proc.is_ok(), "Could not load program \"yes\"");

    drop(proc.unwrap());
}