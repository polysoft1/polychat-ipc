use std::{
    ffi::OsStr,
    path::Path
};
use log::{error, warn, debug};
use walkdir::{DirEntry, WalkDir};

use crate::process_management::process::Process;

#[cfg(target_os = "windows")]
const EXEC_EXTENSION: &str = "exe";
#[cfg(not(target_os = "windows"))]
const EXEC_EXTENSION: &str = "";

#[derive(Debug)]
pub struct ProcessManager {
    process_list: Vec<Process>,
}

impl ProcessManager {
    pub fn new(path: &str) -> Result<ProcessManager, &str> {
        ProcessManager::from_path(Path::new(path))
    }

    pub fn from_path(dir: &Path) -> Result<ProcessManager, &str> {
        check_directory(dir)?;

        let dir_walk = WalkDir::new(dir).max_depth(2).min_depth(2).follow_links(false);
        let mut exec_vec: Vec<Process> = Vec::new();

        for entry in dir_walk.into_iter().filter_entry(|e| expected_file(e)) {
            let file = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Could not read directory entry: {}", e);
                    continue;
                }
            };
            
            debug!("Found executable: {}", file.path().display());
            let proc = Process::new(file.path());

            match proc {
                Ok(p) => exec_vec.insert(0, p),
                Err(e) => {
                    warn!("{}", e);
                    continue;
                }
            };
        }

        Ok(ProcessManager {
            process_list: exec_vec
        })
    }
}

fn check_directory(dir: &Path) -> Result<(), &str> {
    let str_path = dir.to_str().unwrap_or("Unknown path");
    if !dir.is_absolute() {
        error!("Path {} is not absolute", str_path);
        return Err("Path must be absolute");
    }
    if !dir.exists() {
        error!("Directory {} does not exist", str_path);
        return Err("Directory does not exist");
    }
    if !dir.is_dir() {
        error!("Path {} is not a directory", str_path);
        return Err("Path is not a directory");
    }

    Ok(())
}

fn expected_file(entry: &DirEntry) -> bool {
    let is_file = entry.file_type().is_file();
    let extension = entry.path().extension().unwrap_or(OsStr::new(""));

    return is_file && extension == EXEC_EXTENSION;
}

#[test]
fn test_loading_from_dir() {
    let proc_man = ProcessManager::new("/home/keith/.polychat/plugins");

    assert!(proc_man.is_ok(), "Could not load processes from folder: {}", proc_man.unwrap_err());
}

#[test]
fn test_loading_from_realitve_path() {
    let proc_man = ProcessManager::new("./plugins");

    assert!(proc_man.is_err(), "Process Manager loaded from realtive path");
}

#[test]
fn test_loading_from_file() {
    let proc_man = ProcessManager::new("/etc/passwd");
    assert!(proc_man.is_err(), "Process Manager loaded from file path");
}