#[cfg(test)]
mod test {
    use rstest::*;
    use serde_json::value::RawValue;

    use tokio_test::assert_ok;

    use polychat_ipc::{
        core::socket_handler::SocketHandler,
        client_sdk::socket::SocketCommunicator,
        api::schema::{
            instructions::{
                CoreInstruction,
                CoreInstructionType,
                PluginInstruction,
                PluginInstructionType
            }
        }
    };

    #[rstest]
    #[case(CoreInstructionType::Init)]
    #[case(CoreInstructionType::KeepaliveResponse)]
    #[case(CoreInstructionType::AuthAccountResponse)]
    #[test_log::test(tokio::test)]
    async fn integration_test_core_instruction_sending(#[case] ins_type: CoreInstructionType){
        let socket_name = format!("int_test_{}", ins_type);
        let mut handler = create_handler(socket_name.clone());

        let mut comm = create_communicator(socket_name).await;
        let instruct = CoreInstruction{
            payload: create_core_payload(),
            instruction_type: ins_type
        };

        let send_res = comm.send_core_instruction(&instruct);

        let recv_res = handler.get_instruction();
        assert_ok!(send_res.await);

        let recv_res = assert_ok!(recv_res.await);
        assert_eq!(instruct, recv_res);
    }

    #[rstest]
    #[case(PluginInstructionType::Keepalive)]
    #[case(PluginInstructionType::AuthAccount)]
    #[test_log::test(tokio::test)]
    async fn integration_test_plugin_instruction_client(#[case] ins_type: PluginInstructionType) {
        let socket_name = format!("client_ins_{}", ins_type);
        let mut server = create_handler(socket_name.clone());
        let mut client = create_communicator(socket_name).await;

        let instruct = PluginInstruction{
            payload: create_core_payload(),
            instruction_type: ins_type
        };

        assert_ok!(server.send_plugin_instruction(&instruct).await);

        let recv = assert_ok!(client.recv_plugin_instruction().await);
        assert_eq!(instruct, recv);
    }

    fn create_handler(name: String) -> SocketHandler {
        assert_ok!(SocketHandler::new(name))
    }

    async fn create_communicator(name: String) -> SocketCommunicator {
        assert_ok!(SocketCommunicator::new(name).await)
    }

    fn create_core_payload() -> Box<RawValue> {
        let payload = r#"{}"#;
        serde_json::value::RawValue::from_string(payload.to_string()).unwrap()
    }
}