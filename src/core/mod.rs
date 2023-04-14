pub mod ui_interface;

use std::{path::PathBuf, env};

use anyhow::Result;

use crate::process_management::process_manager::ProcessManager;

use self::ui_interface::ui_trait::GUI;

pub struct Core {
    proc_manager: ProcessManager
}

/**
 * The component that is run by the UI to enable PolyChat to work.
 */
impl Core {
    /**
     * Creates a new Core object with a plugin dir in $HOME/.polychat/plugins
     * The configs will be stored in the app data directory (%appdata% or ~/.local/share)
     * 
     * # Returns
     * A valid `Core` object on success
     * A string describing the error on failure (more details can be found in logs, adjust `RUST_LOG` level)
     */
    pub fn new_in_home() -> Result<Core> {
        let home_dir = dirs::home_dir();
        if home_dir.is_none() {
            return Err(anyhow::Error::msg("No home dir found"));
        }
        let mut plugin_dir = home_dir.unwrap();
        plugin_dir.push(".polychat");
        Self::new_from_dir(plugin_dir)
    }

    /**
     * Creates a new Core object with a plugin dir in $WORKDIR/polychat/plugins
     * The configs will be stored in the app data directory (%appdata% or ~/.local/share)
     * The configs may be changed to the working dir, too.
     * 
     * # Returns
     * A valid `Core` object on success
     * A string describing the error on failure (more details can be found in logs, adjust `RUST_LOG` level)
     */
    pub fn new_in_working_dir() -> Result<Core> {
        let working_dir = env::current_dir();
        let mut plugin_dir = working_dir.expect("Could not get working directory.");
        plugin_dir.push("polychat");
        Self::new_from_dir(plugin_dir)
    }

    /**
     * Creates a new Core object that uses the given directory.
     * The plugin directory will be dir/plugins
     * 
     * # Returns
     * A valid `Core` object on success
     * A string describing the error on failure (more details can be found in logs, adjust `RUST_LOG` level)
     */
    pub fn new_from_dir(mut dir: PathBuf) -> Result<Core> {
        dir.push("plugins");
        let man = ProcessManager::from_dir_path(dir)?;

        Ok(Core {
            proc_manager: man
        })
    }

    pub fn run(&mut self, ui: &dyn GUI) -> Result<()> {
        ui.on_core_pre_init();
        self.proc_manager.prepare_dir()?;
        let load_process_result = self.proc_manager.load_processes(Some(ui));
        if load_process_result.is_err() {
            ui.on_plugin_load_failure(load_process_result.unwrap_err().to_string())
        }
        ui.on_core_post_init(self.proc_manager.get_dir());
        // TODO: Run loop for receiving from plugins.
        Ok(())
    }
}
