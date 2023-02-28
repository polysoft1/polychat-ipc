#[cfg(test)]
mod test {
    use polychat_ipc::{core::socket_handler::SocketHandler, polychat_plugin_sdk_rust::socket::SocketCommunicator, api::schema::instructions::SerializableCoreInstr};
    use claims::{assert_ok, assert_some};
    use polychat_ipc::{
        api::schema::instructions::{CoreInstructionType, PluginInstructionType},
        process_management::process::Process
    };
    use rstest::*;
    use serde_json::value::RawValue;

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
        let mut proc = assert_ok!(Process::new(TEST_PROGRAM, create_socket_server(&name)));
        let mut comms = create_socket_client(&name).await;

        let core_payload = SerializableCoreInstr {
            instruction_type: ins_type,
            payload: create_core_payload()
        };

        assert_ok!(comms.send_core_instruction(&core_payload).await);
        let recv_data = assert_some!(assert_ok!(proc.get_next_instruction().await));
        
        assert_eq!(core_payload, recv_data.into());
    }

    pub fn create_socket_server(name: &String) -> SocketHandler {
        assert_ok!(SocketHandler::new(name))
    }

    async fn create_socket_client(name: &String) -> SocketCommunicator {
        assert_ok!(SocketCommunicator::new(name).await)
    }
    
    fn create_core_payload() -> Box<RawValue> {
        let payload = r#"{}"#;
        serde_json::value::RawValue::from_string(payload.to_string()).unwrap()
    }
}