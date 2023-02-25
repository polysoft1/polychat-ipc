use super::schema::{
    auth::AuthAccountInstruction,
    keepalive::KeepaliveInstruction,
    instructions::{PluginInstructionType, DeserializablePluginInstr}
};

use anyhow::Result;

use log::{trace, error};
use std::sync::Arc;

/// A trait to be implemented by the plugin for instructions sent from the
/// core to the plugin.
pub trait PluginInstructionHandler {
    fn on_keepalive(&self, data: KeepaliveInstruction);
    fn on_auth_account(&self, data: AuthAccountInstruction);
}

/// A function that finishes processing the PluginInstruction, and sends the
/// data to the correct function on the given handler function.
pub fn call_core_handler(unprocessed_instr: &DeserializablePluginInstr,
    interface: Arc<dyn PluginInstructionHandler>) -> Result<()>
{
    match unprocessed_instr.instruction_type {
        PluginInstructionType::AuthAccount => {
            match serde_json::from_str::<AuthAccountInstruction>(unprocessed_instr.payload.get()) {
                Ok(data) => {
                    trace!("Got valid data for AuthAccountInstruction. Calling handler function.");
                    interface.as_ref().on_auth_account(data);
                    Ok(())
                },
                Err(e) => {
                    error!("Invalid data for instruction type AuthAccountInstruction.");
                    return Err(e.into());
                }
            }

        },
        PluginInstructionType::Keepalive => {
            match serde_json::from_str::<KeepaliveInstruction>(unprocessed_instr.payload.get()) {
                Ok(data) => {
                    trace!("Got valid data for init. Calling handler function.");
                    interface.as_ref().on_keepalive(data);
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
