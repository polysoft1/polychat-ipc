// Tests that utilize the test plugin binary.
// These tests are useful because they can test all features end-to-end.

#[cfg(test)]
mod test {
    use polychat_ipc::{core::socket_handler::SocketHandler, api::schema::{instructions::CoreInstructionType, protocol::InitDataInstruction}};
    use rstest::*;
    use tokio_test::assert_ok;
    use std::process::Command;
    use assert_cmd::prelude::*; // Add methods on command
    use log::debug;

    /// This function:
    /// - Starts the necessary components of the core
    /// - Starts the plugin
    /// - Verifies that the plugin sent the Init core instruction.
    #[rstest]
    #[test_log::test(tokio::test)]
    async fn integration_test_plugin_init() {
        // Start the component from core that starts the IPC connections.
        debug!("Starting socket");
        let socket_name = format!("test_plugin_test_init");
        let mut handler = create_handler(socket_name.clone());
        
        // Start the plugin
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

}