use std::env;

use crate::api::schema::{instructions::{CoreInstructionType, SerializableCoreInstr}, protocol::{InitDataInstruction, Version, ProtocolData}};
use log::error;
use super::socket::SocketCommunicator;

// A blocking function that determines the socket name from command line args,
// then opens the socket.
pub async fn run_plugin() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 1 {
        panic!("Incorrect number of args while running plugin. Got {}, expected 1.", args.len());
    }
    let socket_id = args[0].clone();

    let ipc_connection = SocketCommunicator::new(socket_id).await;
    match ipc_connection {
        Ok(mut connection) => {
            // TODO: Make it so the plugin passes this in instead of using example data.
            let instr_payload = InitDataInstruction {
                api_version: Version {major: 0, minor: 1, patch: 0},
                plugin_version: Version {major: 0, minor: 1, patch: 0},
                protocol_data: ProtocolData { protocol_service_name: "example_protocol".to_string(), auth_methods: vec![] },
            };
            let init_instr = SerializableCoreInstr {
                instruction_type: CoreInstructionType::Init,
                payload: instr_payload,
            };
            let send_result = connection.send_core_instruction(&init_instr).await;
            if send_result.is_err() {
                error!("Error while trying to send core instruction: {:?}", send_result.err())
            }
        },
        Err(e) => {
            panic!("Error while opening IPC connection. Error: {}", e);
        }
    }
}