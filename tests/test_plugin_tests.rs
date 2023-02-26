// Tests that utilize the test plugin binary.
// These tests are useful because they can test all features end-to-end.

#[cfg(test)]
mod test {
    use polychat_ipc::{core::socket_handler::SocketHandler};
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
        let handler = create_handler(socket_name.clone());
        
        // Start the plugin
        debug!("Starting plugin");
        let mut cmd = Command::cargo_bin("test-plugin").unwrap();
        cmd.arg(socket_name.clone());
        let plugin_output = cmd.unwrap();
        debug!("Output of plugin: {:?}", plugin_output);
        debug!("Started plugin");

        // Process the connection
        let conn = assert_ok!(handler.get_connection().await);
        debug!("Received connection");

        // Await the init instruction
        let recv_res = handler.get_core_instruction_data(conn);
        let recv_res = assert_ok!(recv_res.await);
        debug!("Received data: {}", recv_res);
        assert_ok!(handler.handle_recv_core_message(recv_res).await);
        debug!("Done");
    }

    fn create_handler(name: String) -> SocketHandler {
        assert_ok!(SocketHandler::new(name))
    }

}