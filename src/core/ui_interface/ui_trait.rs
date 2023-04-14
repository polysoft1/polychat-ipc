use std::path::PathBuf;

use crate::api::schema::protocol::InitDataInstruction;

use super::load_status::LoadStatus;


pub trait GUI {
    // ----------------------------- //
    // Core
    // ----------------------------- //

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


    // ----------------------------- //
    // Core's management of plugins
    // ----------------------------- //

    /**
     * Called when a plugin's process is loaded by the core
     * This is separate from when the plugin states that it is initialized.
     * 
     * Do not assume that this will come before the plugin init
     * instruction, because they can run in parallel.
     */
    fn on_plugin_loaded(&self, plugin_name: String); // TODO: Switch to a more useful data type. Likely struct.

    /**
     * Called when it fails to load a plugin.
     */
    fn on_plugin_load_failure(&self, error_msg: String); // TODO: Switch to a more useful data type. Maybe enum.

    /**
     * Called when the plugins loaded status switches to:
     * - Started
     * - Finished
     */
    fn on_plugins_loaded_status_change(&self, status: LoadStatus);

    // ----------------------------- //
    // Plugins events sent from plugins
    // ----------------------------- //

    /**
     * Called when a plugin sends the init instruction.
     * 
     * Do not assume that this will come before or after the plugin process loaded
     * call, because they can run in parallel.
     */
    fn on_plugin_init(&self, plugin_init_data: InitDataInstruction);

}