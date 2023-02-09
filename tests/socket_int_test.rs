#[cfg(test)]
mod test {
    use rstest::*;
    use serde_json::value::RawValue;
    use log::debug;

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
        let handler = create_handler(socket_name.clone());

        let mut comm = create_communicator(socket_name).await;
        let instruct = CoreInstruction{
            payload: create_core_payload(),
            instruction_type: ins_type
        };

        let send_res = comm.send_core_instruction(&instruct);

        let conn = handler.get_connection().await;
        assert!(conn.is_ok(), "Could not get connection from socket: {}", conn.unwrap_err());
        let recv_res = handler.get_core_instruction_data(conn.unwrap());
        send_res.await;

        let recv_res = recv_res.await;
        assert!(recv_res.is_ok(), "Error receiving CoreInstruction via Core");
        let des_ins = handler.handle_message(recv_res.unwrap()).await;
        assert!(des_ins.is_ok(), "Error Decoding CoreInstruction");

        assert_eq!(des_ins.unwrap(), instruct);
    }

    #[rstest]
    #[case(PluginInstructionType::Keepalive)]
    #[case(PluginInstructionType::AuthAccount)]
    #[test_log::test(tokio::test)]
    async fn integration_test_plugin_instruction_client(#[case] ins_type: PluginInstructionType) {
        let socket_name = format!("client_ins_{}", ins_type);
        let server = create_handler(socket_name.clone());
        let mut client = create_communicator(socket_name).await;

        let instruct = PluginInstruction{
            payload: create_core_payload(),
            instruction_type: ins_type
        };
        let conn = server.get_connection().await;
        assert!(conn.is_ok(), "Could not accept a connection!");
        let send = server.send_plugin_instruction(conn.unwrap(), &instruct).await;
        assert!(send.is_ok(), "Issue sending plugin instruction!");

        let recv = client.recv_plugin_instruction().await;
        assert!(recv.is_ok(), "Issue receving plugin instruction!");
        assert_eq!(instruct, recv.unwrap());
    }

    fn create_handler(name: String) -> SocketHandler {
        let handler = SocketHandler::new(name);
        assert!(handler.is_ok(), "Could not initialize SocketHandler");

        handler.unwrap()
    }
    async fn create_communicator(name: String) -> SocketCommunicator {
        let comms = SocketCommunicator::new(name).await;
        assert!(comms.is_ok(), "Could not initialize SocketCommunicator: {}", comms.unwrap_err());

        comms.unwrap()
    }

    fn create_core_payload() -> Box<RawValue> {
        let payload = r#"{}"#;
        serde_json::value::RawValue::from_string(payload.to_string()).unwrap()
    }
}