use std::fmt::{Display, Debug};
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

/// An instruction to be sent from plugin to core.
#[derive(Serialize, Debug)]
pub struct SerializableCoreInstr<P: Serialize + Debug> {
    pub instruction_type: CoreInstructionType,
    // If further optimization is desired, you can use: #[serde(borrow)]
    // But this comes at the cost of needing to ensure the String that is used
    // to create this struct has a lifetime that matches or exceeds this.
    pub payload: P,
}

/// An instruction to be sent from core to plugin.
#[derive(Serialize, Debug)]
pub struct SerializablePluginInstr<P: Serialize + Debug> {
    pub instruction_type: PluginInstructionType,
    // If further optimization is desired, you can use: #[serde(borrow)]
    // But this comes at the cost of needing to ensure the String that is used
    // to create this struct has a lifetime that matches or exceeds this.
    pub payload: P,
}
/// An instruction that was sent from plugin to core
#[derive(Deserialize, Debug)]
pub struct DeserializableCoreInstr {
    pub instruction_type: CoreInstructionType,
    // If further optimization is desired, you can use: #[serde(borrow)]
    // But this comes at the cost of needing to ensure the String that is used
    // to create this struct has a lifetime that matches or exceeds this.
    pub payload: Box<RawValue>,
}

/// An instruction that was sent from core to plugin
#[derive(Deserialize, Debug)]
pub struct DeserializablePluginInstr {
    pub instruction_type: PluginInstructionType,
    // If further optimization is desired, you can use: #[serde(borrow)]
    // But this comes at the cost of needing to ensure the String that is used
    // to create this struct has a lifetime that matches or exceeds this.
    // Can also use &'a instead of Box
    pub payload: Box<RawValue>,
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

impl<P: Serialize + Debug> PartialEq for SerializableCoreInstr<P> {
    fn eq(&self, other: &Self) -> bool {
        let serialized_payload_1 = serde_json::to_string(&self.payload);
        let serialized_payload_2 = serde_json::to_string(&other.payload);
        if serialized_payload_1.is_err() || serialized_payload_2.is_err() {
            return false;
        }
        let payloads_equal = serialized_payload_1.unwrap() == serialized_payload_2.unwrap();
        let ins_equal = self.instruction_type == other.instruction_type;

        ins_equal && payloads_equal
    }
}

impl<P: Serialize + Debug> PartialEq for SerializablePluginInstr<P> {
    fn eq(&self, other: &Self) -> bool {
        let serialized_payload_1 = serde_json::to_string(&self.payload);
        let serialized_payload_2 = serde_json::to_string(&other.payload);
        if serialized_payload_1.is_err() || serialized_payload_2.is_err() {
            return false;
        }
        let payloads_equal = serialized_payload_1.unwrap() == serialized_payload_2.unwrap();
        let ins_equal = self.instruction_type == other.instruction_type;

        ins_equal && payloads_equal
    }
}

impl From<DeserializableCoreInstr> for SerializableCoreInstr<Box<RawValue>> {
    fn from(value: DeserializableCoreInstr) -> Self {
        SerializableCoreInstr { instruction_type: value.instruction_type, payload: value.payload }
    }
}

impl From<DeserializablePluginInstr> for SerializablePluginInstr<Box<RawValue>> {
    fn from(value: DeserializablePluginInstr) -> Self {
        SerializablePluginInstr { instruction_type: value.instruction_type, payload: value.payload }
    }
}

impl Clone for DeserializableCoreInstr {
    fn clone(&self) -> Self {
        DeserializableCoreInstr { instruction_type: self.instruction_type.clone(), payload: self.payload.clone() }
    }
}

impl Clone for CoreInstructionType {
    fn clone(&self) -> Self {
        match self {
            CoreInstructionType::Init => CoreInstructionType::Init,
            CoreInstructionType::KeepaliveResponse => CoreInstructionType::KeepaliveResponse,
            CoreInstructionType::AuthAccountResponse => CoreInstructionType::AuthAccountResponse
        }
    }
}
