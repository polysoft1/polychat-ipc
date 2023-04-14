use std::{
    ffi::OsStr,
    path::{PathBuf, Path}, str::FromStr, fs,
};
use log::{error, warn, debug};
use scopeguard::defer;
use tokio::sync::mpsc::{self, Receiver, Sender};
use walkdir::{DirEntry, WalkDir};
use anyhow::Result;
use rand::{
    distributions::Alphanumeric,
    thread_rng, Rng
};

use crate::{process_management::{
    process::Process,
    error::ProcessManagerError,
    ipc_server::IPCServer,
}, core::{ui_interface::{ui_trait::GUI, load_status::LoadStatus}}, api::schema::instructions::DeserializableCoreInstr};

#[cfg(target_os = "windows")]
const EXEC_EXTENSION: &str = "exe";
#[cfg(not(target_os = "windows"))]
const EXEC_EXTENSION: &str = "";

#[derive(Debug)]
pub struct ProcessManager {
    dir: Option<PathBuf>,
    loaded_processes: Vec<Process>,
    shared_queue_rx: Receiver<DeserializableCoreInstr>,
    shared_queue_tx: Sender<DeserializableCoreInstr>,
}

impl ProcessManager {
    /**
     * Creates a new ProcessManager.
     * Does not attempt to load any processes from executables.
     * 
     * You can still load executables afterwards with load_executable.
     */
    pub fn new() -> ProcessManager {
        let (tx, rx) = mpsc::channel(100);
        ProcessManager {
            dir: None,
            loaded_processes: vec![],
            shared_queue_rx: rx,
            shared_queue_tx: tx,
        }
    }

    /**
     * Creates a new ProcessManager with a string slice
     * 
     * # Required Arguments
     * ## path
     * The absolute path to a directory containing a set of directories which contain executables
     *
     * # Returns
     * A ProcessManager on success
     * 
     * A string slice describing the error on failure (check logs for more details)
     */
    pub fn from_dir_str(path: &str) -> Result<ProcessManager> {
        ProcessManager::from_dir_path(PathBuf::from_str(path)?)
    }

    /**
     * Creates a new ProcessManager from a Path
     * 
     * # Required Arguments
     * ## dir
     * The absolute path to a directory containing a set of directories which contain executables
     * 
     * # Returns
     * A ProcessManager on success
     * 
     * A string slice describing the error on failure (check logs for more details)
     */
    pub fn from_dir_path(dir: PathBuf) -> Result<ProcessManager> {
        // Validate
        check_directory(dir.clone())?;
        // Create manager
        let mut manager = Self::new();
        manager.dir = Some(dir);
        Ok(manager)
    }

    pub fn get_dir(&self) -> Option<PathBuf> {
        self.dir.clone()
    }

    /**
     * If the dir is set, it ensures that it contains the necessary folder.
     * Will also initialize parent directories if necessary.
     * 
     * Returns ProcessManagerError::NoPath error if no path is set.
     */
    pub fn prepare_dir(&self) -> Result<()> {
        match self.dir.clone() {
            None => {
                let err = ProcessManagerError::NoPath;
                error!("{}", err);
                Err(anyhow::Error::new(err))
            },
            Some(dir) => {
                Ok(fs::create_dir_all(dir)?)
            }
        }
    }

    /**
     * Loads the processes in the previously set directory.
     * Calls the GUI for each plugin loaded and for each failure.
     * 
     * If no dir is set, returns ProcessManagerError::NoPath Result
     * 
     * # Platform Dependent Behavior
     * - Windows: Expects `.exe` to be the extension of the executables
     * - Everything Else: Expects no extension on the executables
     */
    pub fn load_processes(&mut self, ui: Option<&dyn GUI>) -> Result<()> {
        if let Some(ui) = ui {
            ui.on_plugins_loaded_status_change(LoadStatus::Started)
        }
        defer! { // Run finished once this function finishes.
            if let Some(ui) = ui {
                ui.on_plugins_loaded_status_change(LoadStatus::Finished)
            }
        }

        let dir: PathBuf;
        match self.dir.clone() {
            None => {
                let err = ProcessManagerError::NoPath;
                error!("{}", err);
                return Err(anyhow::Error::new(err));
            },
            Some(manager_dir) => {
                check_directory(manager_dir.clone())?;
                dir = manager_dir;
            }
        }
        println!("Loading plugins from dir {:?}", dir);
        debug!("Loading plugins from dir {:?}", dir);

        let dir_walk = WalkDir::new(dir).max_depth(2).min_depth(2).follow_links(false);

        for entry in dir_walk.into_iter().filter_entry(|e| expected_file(e)) {
            let file = match entry {
                Ok(e) => e,
                Err(e) => {
                    // Should this error get sent to the GUI?
                    warn!("Could not read directory entry: {}", e);
                    continue;
                }
            };

            debug!("Found executable: {}", file.path().display());
            let load_process_result = self.load_process(file.path());
            match load_process_result {
                Ok(_) => {
                    if let Some(ui) = ui {
                        ui.on_plugin_loaded(file.file_name().to_string_lossy().into_owned());
                    }
                },
                Err(e) => {
                    if let Some(ui) = ui {
                        ui.on_plugin_load_failure(e.to_string())
                    }
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    // Loads the process, and returns the filename
    pub fn load_process(&mut self, path: &Path) -> Result<()> {
        let socket_name: String = generate_random_ipc_id();
        let socket = IPCServer::new(socket_name)?;
        
        let proc = Process::new(path, socket, self.shared_queue_tx.clone());

        match proc {
            Ok(p) => {
                self.loaded_processes.push(p);
            },
            Err(e) => {
                warn!("{}", e);
                return Err(e);
            }
        };
        Ok(())
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
fn check_directory(dir: PathBuf) -> Result<(), ProcessManagerError> {
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

fn generate_random_ipc_id() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(7).map(char::from).collect()
}

#[cfg(test)]
mod test{
    use crate::process_management::process_manager::ProcessManager;

    // The Ok tests will be done in the integration tests with a plugin binary.
    use claims::assert_err;

    #[test]
    fn test_loading_from_relative_path() {
        assert_err!(ProcessManager::from_dir_str("./plugins"));
    }

    #[test]
    fn test_loading_from_file() {
        assert_err!(ProcessManager::from_dir_str("/etc/passwd"));
    }
}
