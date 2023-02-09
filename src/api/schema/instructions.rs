use std::fmt::Display;
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
pub struct CoreInstruction {
    pub instruction_type: CoreInstructionType,
    pub payload: Box<RawValue>, // or &'a RawValue
}

/// An instruction that was sent from plugin to core
#[derive(Serialize, Deserialize, Debug)]
pub struct PluginInstruction {
    pub instruction_type: PluginInstructionType,
    pub payload: Box<RawValue>, // or &'a RawValue
}

impl Display for CoreInstructionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreInstructionType::Init => write!(f, "Init"),
            CoreInstructionType::KeepaliveResponse => write!(f, "KeepaliveResponse"),
            CoreInstructionType::AuthAccountResponse => write!(f, "AuthAccountResponse")
        }
    }
}

impl Display for PluginInstructionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginInstructionType::AuthAccount => write!(f, "AuthAccount"),
            PluginInstructionType::Keepalive => write!(f, "KeepAlive")
        }
    }
}

impl PartialEq for PluginInstruction {
    fn eq(&self, other: &Self) -> bool {
        let payloads_equal = self.payload.to_string() == other.payload.to_string();
        let ins_equal = self.instruction_type == other.instruction_type;

        ins_equal && payloads_equal
    }
}

impl PartialEq for CoreInstruction {
    fn eq(&self, other: &Self) -> bool {
        let payloads_equal = self.payload.to_string() == other.payload.to_string();
        let ins_equal = self.instruction_type == other.instruction_type;

        ins_equal && payloads_equal
    }
}

impl Display for CoreInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Type: {}, Payload: {}", self.instruction_type, self.payload.get())
    }
}

impl Display for PluginInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Type: {}, Payload: {}", self.instruction_type, self.payload.get())
    }
}