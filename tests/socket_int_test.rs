#[cfg(test)]
mod test {
    use futures::join;
    use rstest::*;
    use serde_json::value::RawValue;
    use log::debug;

    use polychat_ipc::{
        core::socket_handler::SocketHandler,
        client_sdk::socket::SocketCommunicator,
        api::schema::{
            instructions::{CoreInstruction, CoreInstructionType},
            protocol::*
        }
    };

    #[rstest]
    #[case(CoreInstructionType::Init)]
    #[case(CoreInstructionType::KeepaliveResponse)]
    #[case(CoreInstructionType::AuthAccountResponse)]
    #[test_log::test(tokio::test)]
    async fn integration_test_core_instruction_sending(#[case] ins_type: CoreInstructionType){
        let socket_name = format!("int_test_{}", ins_type);
        debug!("Creating SocketHandler {}", socket_name);
        let handler = create_handler(socket_name.clone());

        debug!("Creating SocketCommunicator at {}", socket_name);
        let mut comm = create_communicator(socket_name).await;
        let instruct = CoreInstruction{
            payload: create_core_payload(),
            instruction_type: ins_type
        };

        debug!("Sending data to socket");
        let send_res = comm.send_core_message(&instruct);

        debug!("Receiving data from socket");
        let recv_res = handler.get_data_from_new_conn();
        send_res.await;
        comm.close().await;

        let recv_res = recv_res.await;
        assert!(recv_res.is_ok(), "Error receiving CoreInstruction via Core");
        let des_ins = handler.handle_message(recv_res.unwrap()).await;
        assert!(des_ins.is_ok(), "Error Decoding CoreInstruction");

        assert_eq!(des_ins.unwrap(), instruct);
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