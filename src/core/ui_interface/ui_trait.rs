use std::path::PathBuf;

use super::load_status::LoadStatus;


pub trait GUI {
    /**
     * Called early on in the core init phase.
     */
    fn on_core_pre_init(&self);

    /**
     * Called after the core init phase.
     * 
     * @param: plugin_loaded_dir: The dir where searched to load plugins, if any.
     */
    fn on_core_post_init(&self, plugin_loaded_dir: Option<PathBuf>);

    /**
     * Called when a plugin is loaded by the core
     */
    fn on_plugin_loaded(&self, plugin_name: String); // TODO: Switch to a more useful data type. Likely struct.

    /**
     * Called when it fails to load a plugin.
     */
    fn on_plugin_load_failure(&self, error_msg: String); // TODO: Switch to a more useful data type. Maybe enum.

    /**
     * Called when the plugin load status switches to:
     * - Started
     * - Finished
     */
    fn on_plugin_loaded_status_change(&self, status: LoadStatus);

}