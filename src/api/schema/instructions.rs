use serde::{Serialize, Deserialize};
use serde_json::value::RawValue;

/// An enum for every instruction that can be sent from the plugin to the core
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum CoreInstructionType {
    Init,
    KeepaliveResponse,
    AuthAccountResponse,
}

/// An enum for every instruction that can be sent from the core to the plugin
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum PluginInstructionType {
    Keepalive,
    AuthAccount,
}

/// An instruction that was sent from plugin to core
#[derive(Serialize, Deserialize, Debug)]
struct CoreInstruction {
    pub instruction_type: CoreInstructionType,
    pub payload: Box<RawValue>, // or &'a RawValue
}


#[cfg(test)]
mod tests {
    

}