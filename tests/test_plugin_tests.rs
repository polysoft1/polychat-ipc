// Tests that utilize the test plugin binary.
// These tests are useful because they can test all features end-to-end.

#[cfg(test)]
mod test {
    use polychat_ipc::{core::socket_handler::SocketHandler, api::schema::{instructions::CoreInstructionType, protocol::InitDataInstruction}, process_management::process_manager::ProcessManager};
    use rstest::*;
    use claims::assert_ok;
    use std::process::Command;
    use assert_cmd::prelude::*; // Add methods on command
    use log::debug;
    use std::path::PathBuf;
    use testdir::testdir;

    /**
     * This function:
     * - Starts the necessary components of the core
     * - Starts the plugin
     * - Verifies that the plugin sent the Init core instruction.
     */
    #[rstest]
    #[test_log::test(tokio::test)]
    async fn integration_test_plugin_init() {
        // Start the component from core that starts the IPC connections.
        debug!("Starting socket");
        let socket_name = format!("test_plugin_test_init");
        let mut handler = create_handler(socket_name.clone());
        
        // Start the plugin
        // Does not use ProcessManager in order to isolate this test to the plugin itself.
        debug!("Starting plugin");
        let mut cmd = Command::cargo_bin("test-plugin").unwrap();
        cmd.arg(socket_name.clone());
        let plugin_output = cmd.unwrap();
        debug!("Output of plugin: {:?}", plugin_output);
        debug!("Started plugin. Now receiving instruction from plugin.");

        // Await the init instruction
        let recv_res = handler.get_instruction();
        let recv_res = assert_ok!(recv_res.await);
        assert_eq!(CoreInstructionType::Init, recv_res.instruction_type);
        debug!("Received Init. Now validating that it can deserialize it.");
        let deserialized_instr = serde_json::from_str::<InitDataInstruction>(recv_res.payload.get());
        assert_ok!(deserialized_instr);
        debug!("Done");
    }

    fn create_handler(name: String) -> SocketHandler {
        assert_ok!(SocketHandler::new(name))
    }

    /**
     * This function builds a valid plugin dir, then runs the process manager with it.
     *
     * Therefore, this tests the ability to traverse the plugin dir, the ability to load
     * plugins, and the ability for those plugins to connect to their IPC sockets/pipes.
     */
    #[rstest]
    #[case(1)]
    #[case(3)]
    #[test_log::test(tokio::test)]
    async fn integration_test_process_manager_load_dir(#[case] plugin_count: i32) {
        let test_plugin_binary = assert_cmd::cargo::cargo_bin("test-plugin");

        let plugins_dir: PathBuf = testdir!(); // This will create the folder, so no need to try.
        debug!("Plugins folder is located at {:?}", plugins_dir);
        // Now create a sub-folder in the plugin_dir for plugins
        for i in 0..plugin_count {
            let plugin_dir = plugins_dir.join(format!("plugin{}", i));
            debug!("Creating test plugin folder {:?}", &plugin_dir);
            assert_ok!(std::fs::create_dir(&plugin_dir));
            debug!("Adding test plugin at \"{:?}\" to plugin folder at \"{:?}\"", &test_plugin_binary, &plugin_dir);
            let exe_in_plugin_dir = plugin_dir.join("test_plugin.exe"); // exe shouldn't matter on unix
            assert_ok!(std::fs::copy(&test_plugin_binary, &exe_in_plugin_dir));
        }
        debug!("Created test plugin dir. Testing loading from the path.");
        assert_ok!(ProcessManager::from_dir_path(plugins_dir));
    }

    /**
     * This function tests loading a single process with the plugin manager.
     * Since it uses a real plugin, it also tests its ability to connect to an IPC socket/pipe.
     */
    #[rstest]
    #[test_log::test(tokio::test)]
    async fn integration_test_process_manager_load_file() {
        let test_plugin_binary = assert_cmd::cargo::cargo_bin("test-plugin");

        // Just load the single plugin directly.
        let mut process_manager = ProcessManager::new();
        assert_ok!(process_manager.load_process(&test_plugin_binary));
    }

}