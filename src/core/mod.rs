pub mod socket_handler;

use anyhow::Result;

use crate::process_management::process_manager::ProcessManager;

pub struct Core {
    proc_manager: ProcessManager
}

impl Core {
    /**
     * Creates a new Core object
     * 
     * # Returns
     * A valid `Core` object on success
     * A string describing the error on failure (more details can be found in logs, adjust `RUST_LOG` level)
     */
    pub fn new() -> Result<Core> {
        let man = ProcessManager::from_dir_str("polychat")?;

        Ok(Core {
            proc_manager: man
        })
    }
}
