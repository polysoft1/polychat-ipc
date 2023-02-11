use std::{
    ffi::OsStr,
    path::Path
};
use log::{error, warn, debug};
use walkdir::{DirEntry, WalkDir};
use anyhow::Result;

use crate::process_management::{
    process::Process,
    error::ProcessManagerError
};

#[cfg(target_os = "windows")]
const EXEC_EXTENSION: &str = "exe";
#[cfg(not(target_os = "windows"))]
const EXEC_EXTENSION: &str = "";

#[derive(Debug)]
pub struct ProcessManager {
    process_list: Vec<Process>,
}

impl ProcessManager {
    /**
     * Creates a new ProcessManager with a string slice
     * 
     * # Arguments
     * ## path
     * The path to a directory containing a set of directories which contain executables
     *
     * # Returns
     * A ProcessManager on success
     * 
     * A string slice describing the error on failure (check logs for more details)
     * 
     * # Platform Dependent Behavior
     * - Windows: Expects `.exe` to be the extension of the executables
     * - Everything Else: Expects no extension on the executables
     */
    pub fn new(path: &'static str) -> Result<ProcessManager> {
        ProcessManager::from_path(Path::new(path))
    }

    /**
     * Creates a new ProcessManager from a Path
     * 
     * # Arguments
     * ## dir
     * The absolute path to a directory containing a set of directories which contain executables
     * 
     * # Returns
     * A ProcessManager on success
     * 
     * A string slice describing the error on failure (check logs for more details)
     * 
     * # Platform Dependent Behavior
     * - Windows: Expects `.exe` to be the extension of the executables
     * - Everything Else: Expects no extension on the executables
     */
    pub fn from_path(dir: &'static Path) -> Result<ProcessManager> {
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

/**
 * Checks a path to see if it meets the following criteria
 * - Is an absolute path
 * - It exists
 * - It is a directory
 * 
 * # Arguments
 * ## dir
 * The path to apply the criteria to
 * 
 * ## Returns
 * Nothing on success
 * 
 * A string slice describing why it does not meet all criteria on failure
 */
fn check_directory(dir: &Path) -> Result<(), ProcessManagerError> {
    if !dir.is_absolute() {
        let err = ProcessManagerError::RelativePath(dir);
        error!("{}", err);
        return Err(err);
    }
    if !dir.exists() {
        let err = ProcessManagerError::NonExistent(dir); 
        error!("{}", err);
        return Err(err);
    }
    if !dir.is_dir() {
        let err = ProcessManagerError::NonDirectory(dir);
        error!("{}", err);
        return Err(err);
    }

    Ok(())
}

/**
 * Applies the following criteria to a [DirEntry]
 * - It is a file
 * - It has an extension indicative of an executable (defined by [EXEC_EXTENSION])
 * 
 * # Arguments
 * ## entry
 * The [DirEntry] to be checked
 * 
 * # Returns
 * A boolean value indicating wheter or not the file is an executable
 */
fn expected_file(entry: &DirEntry) -> bool {
    let is_file = entry.file_type().is_file();
    let extension = entry.path().extension().unwrap_or(OsStr::new(""));

    return is_file && extension == EXEC_EXTENSION;
}

#[cfg(test)]
mod test{
    use crate::process_management::process_manager::ProcessManager;
    use tokio_test::{assert_ok, assert_err};

    #[test]
    #[ignore]
    fn test_loading_from_dir() {
        assert_ok!(ProcessManager::new("C:\\Users\\Admin\\Documents\\polychat-plugins"));
    }

    #[test]
    fn test_loading_from_realitve_path() {
        assert_err!(ProcessManager::new("./plugins"));
    }

    #[test]
    fn test_loading_from_file() {
        assert_err!(ProcessManager::new("/etc/passwd"));
    }
}