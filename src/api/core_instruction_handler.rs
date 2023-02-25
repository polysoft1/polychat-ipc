use super::schema::{
    auth::{AuthAccountResponse},
    protocol::InitDataInstruction,
    keepalive::KeepaliveInstruction,
    instructions::{CoreInstructionType, DeserializableCoreInstr},
};

use log::{trace, error};
use std::sync::Arc;

use anyhow::Result;
/// A trait to be implemented by the core for instructions sent from a plugin
/// Also can be implemented by the plugin SDK, which can then be translated
/// to instructions, and back again in the core.
pub trait CoreInstructionHandler {
    fn on_init(&self, data: InitDataInstruction);
    fn on_keepalive_response(&self, response: KeepaliveInstruction);
    fn on_auth_account_response(&self, response: AuthAccountResponse);
}

/// A function that finishes processing the CoreInstruction, and sends the
/// data to the correct function on the given handler function.
pub fn call_core_handler(unprocessed_instr: &DeserializableCoreInstr,
    interface: Arc<dyn CoreInstructionHandler>) -> Result<()>
{
    match unprocessed_instr.instruction_type {
        CoreInstructionType::Init => {
            match serde_json::from_str::<InitDataInstruction>(unprocessed_instr.payload.get()) {
                Ok(data) => {
                    trace!("Got valid data for init. Calling handler function.");
                    interface.as_ref().on_init(data);
                    Ok(())
                },
                Err(e) => {
                    error!("Invalid data for instruction type Init.");
                    return Err(e.into());
                }
            }
        },
        CoreInstructionType::AuthAccountResponse => {
            match serde_json::from_str::<AuthAccountResponse>(unprocessed_instr.payload.get()) {
                Ok(data) => {
                    trace!("Got valid data for auth response. Calling handler function.");
                    interface.as_ref().on_auth_account_response(data);
                    Ok(())
                },
                Err(e) => {
                    error!("Invalid data for instruction type AuthAccountResponse.");
                    return Err(e.into());
                }
            }

        },
        CoreInstructionType::KeepaliveResponse => {
            match serde_json::from_str::<KeepaliveInstruction>(unprocessed_instr.payload.get()) {
                Ok(data) => {
                    trace!("Got valid data for init. Calling handler function.");
                    interface.as_ref().on_keepalive_response(data);
                    Ok(())
                },
                Err(e) => {
                    error!("Invalid data for instruction type Keepalive.");
                    return Err(e.into());
                }
            }

        },
    }
}
