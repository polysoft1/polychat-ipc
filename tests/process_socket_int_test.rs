#[cfg(test)]
mod test {
    use polychat_ipc::{plugin_management::ipc_server::IPCServer, polychat_plugin_sdk_rust::client::IPCClient, api::schema::instructions::{SerializableCoreInstr, SerializablePluginInstr}};
    use claims::{assert_ok, assert_some};
    use polychat_ipc::{
        api::schema::instructions::{CoreInstructionType, PluginInstructionType},
        plugin_management::process::Process
    };
    use rstest::*;
    use serde_json::value::RawValue;

    // TEST_PROGRAM is an executable that can be run on the local system.
    // The purpose of using this instead of a plugin is to test it in isolation.
    // The test_plugin_tests integration tests use the plugin executable.
    #[cfg(target_os = "windows")]
    const TEST_PROGRAM: &str = "calc.exe";
    #[cfg(not(target_os = "windows"))]
    const TEST_PROGRAM: &str = "yes";

    #[rstest]
    #[case(CoreInstructionType::Init)]
    #[case(CoreInstructionType::KeepaliveResponse)]
    #[case(CoreInstructionType::AuthAccountResponse)]
    #[test_log::test(tokio::test)]
    async fn test_recv_core_inst(#[case] ins_type: CoreInstructionType ) {
        let name = format!("polychat_process_recv_core_inst_{}", ins_type);
        // Create the required shared queue
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        // Start an executable that won't crash.
        // The connection to the socket will be tested separately from process execution in this test.
        let _proc = assert_ok!(Process::new(TEST_PROGRAM, create_socket_server(&name), tx));
        // Run the code that plugins usually run to connect to the socket server.
        let mut comms = create_socket_client(&name).await;

        // Send a message from the plugin code to the core code, and verify that it
        // was passed from comms (the plugin code) to proc (the core code).
        let core_payload = SerializableCoreInstr {
            instruction_type: ins_type,
            payload: create_core_payload()
        };

        assert_ok!(comms.send_core_instruction(&core_payload).await);
        let recv_data = assert_some!(rx.recv().await);
        
        // Ensure the data was sent correctly.
        assert_eq!(core_payload, recv_data.into());
    }

    #[rstest]
    #[case(PluginInstructionType::Keepalive)]
    #[case(PluginInstructionType::AuthAccount)]
    #[test_log::test(tokio::test)]
    async fn test_send_plugin_inst(#[case] ins_type: PluginInstructionType) {
        let name = format!("polychat_process_send_plugin_inst_{}", ins_type);
        // Create the required shared queue
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        // Start an executable that won't crash.
        // The connection to the socket will be tested separately from process execution in this test.
        let mut proc = assert_ok!(Process::new(TEST_PROGRAM, create_socket_server(&name), tx));
        // Run the code that plugins usually run to connect to the socket server.
        let mut comms = create_socket_client(&name).await;

        // Send the Init instruction from the plugin code to the core code, and verify
        // that it was passed from comms (the plugin code) to proc (the core code).
        let core_payload = SerializableCoreInstr {
            instruction_type: CoreInstructionType::Init,
            payload: create_core_payload()
        };
        assert_ok!(comms.send_core_instruction(&core_payload).await);
        rx.recv().await; // Receive it back.


        // Now the other way. Send the case's instruction type from the core's code (proc)
        // to the plugin's code (comms).
        let plugin_payload = SerializablePluginInstr {
            instruction_type: ins_type,
            payload: create_core_payload()
        };
        assert_ok!(proc.send_instruction(&plugin_payload).await);
        let recv_data = assert_ok!(comms.recv_plugin_instruction().await);

        // Ensure the data plugin instruction data was sent correctly.
        assert_eq!(plugin_payload, recv_data.into());
    }

    // Used for creating a core socket server
    pub fn create_socket_server(name: &String) -> IPCServer {
        assert_ok!(IPCServer::new(name))
    }

    // Used for creating the socket client, which is plugin SDK code.
    async fn create_socket_client(name: &String) -> IPCClient {
        assert_ok!(IPCClient::new(name).await)
    }
    
    /// Generates an empty RawValue for use in testing everything but payload transfer.
    fn create_core_payload() -> Box<RawValue> {
        let payload = r#"{}"#;
        serde_json::value::RawValue::from_string(payload.to_string()).unwrap()
    }
}