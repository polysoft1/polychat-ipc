/// Defines the structs used to pass data over IPC.
/// These structs are serializeable and deserializeable.

use serde_json::value::RawValue;
use serde::{
    Serialize,
    Deserialize
};

// ------------------------------------------------------------------------ //
// -------------------------------- Keepalive ----------------------------- //
// ------------------------------------------------------------------------ //

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct KeepaliveInstruction {
    pub id: u64,
}

// ------------------------------------------------------------------------ //
// -------------------------------- Init Data ----------------------------- //
// ------------------------------------------------------------------------ //

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ProtocolData {
    /// The well known name of the service that this plugin is designed for.
    pub protocol_service_name: String,
    /// All of the supported ways to authenticate an account
    pub auth_methods: Vec<AuthMethod>,
}

/// Data sent from the plugin to the core once it's initialized
/// Until this is sent, the plugin is considered to be loading.
/// Failure to sent this in a reasonable time represents a failure to load. 
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct InitDataInstruction {
    /// The version of the API this plugin is defined for.
    pub api_version: Version,
    /// The plugin version. For just keeping track of it
    /// Only the newest version should be loaded in the event of accidentally
    /// placing multiple versions of the same plugin in the plugin directory.
    pub plugin_version: Version,
    /// All of the important info about the protocol
    pub protocol_data: ProtocolData,
}

// ------------------------------------------------------------------------ //
// --------------------------------- Auth --------------------------------- //
// ------------------------------------------------------------------------ //

// Associated auth data

/// Represents a way that the user may log in.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthMethod {
    pub name: String,
    /// The fields they can or must input when authenticating.
    /// In the event of anonymous browsing, this can be empty.
    pub fields: Vec<Field>,
}

/// Represents a field type. Used to allow input validation.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum FieldType {
    Integer,
    String,
    Url
}

/// A field in a login method.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub value: Option<String>, // Populated when 
    pub required: bool,
    // Sensitive fields have their value hidden
    pub sensitive: bool
}

/// The possible values for the AuthResult.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum AuthResult {
    Success,
    FailRejected,
    FailConnectionError,
    Connecting
}

// The actual auth instruction and response instruction.

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthAccountInstruction {
    used_authmethod: AuthMethod,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthAccountResponse {
    // TODO: Account ID
    result: AuthResult,
    details: String,
}

// ------------------------------------------------------------------------ //
// ----------------------------- Instructions ----------------------------- //
// ------------------------------------------------------------------------ //

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


// ------------------------------------------------------------------------ //
// ------------------------------ Unit Tests ------------------------------ //
// ------------------------------------------------------------------------ //
#[cfg(test)]
mod tests {
    use super::*;

    // Serialization + Deserialization tests
    // For all of the types, these tests serialize and deserialize them to
    // ensure it behaves as expected
    // To see the serialized structs as json when you run the tests, run it
    // as `cargo test -- --nocapture`
    #[test]
    fn test_keepalive_instruction_serialization() {
        let original = KeepaliveInstruction { id: 0 };
        let serialized = serde_json::to_string(&original).unwrap();

        println!("serialized keepalive = {}", serialized);

        // Convert the JSON string back to a Point.
        let deserialized: KeepaliveInstruction = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_init_data_instruction_serialization() {
        let original = InitDataInstruction {
            api_version: Version { major: 0, minor: 0, patch: 0 },
            plugin_version: Version { major: 0, minor: 0, patch: 0 },
            protocol_data: ProtocolData {
                protocol_service_name: "test".to_string(),
                auth_methods: vec![
                    AuthMethod {
                        name: "test".to_string(),
                        fields: vec![
                            Field {
                                name: "test".to_string(),
                                field_type: FieldType::String,
                                value: None,
                                required: true,
                                sensitive: false,
                            }
                        ]
                    }
                ]
            }

        };
        let serialized = serde_json::to_string(&original).unwrap();

        println!("serialized InitDataInstruction = {}", serialized);

        // Convert the JSON string back to a Point.
        let deserialized: InitDataInstruction = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_auth_account_instruction_serialization() {
        let original = AuthAccountInstruction {
            used_authmethod: AuthMethod {
                name: "test".to_string(),
                fields: vec![],
            },

        };
        let serialized = serde_json::to_string(&original).unwrap();

        println!("serialized AuthAccountInstruction = {}", serialized);

        // Convert the JSON string back to a Point.
        let deserialized: AuthAccountInstruction = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_auth_account_response_serialization() {
        let original = AuthAccountResponse {
            result: AuthResult::Success,
            details: "test".to_string(),
        };
        let serialized = serde_json::to_string(&original).unwrap();

        println!("serialized AuthAccountResponse = {}", serialized);

        // Convert the JSON string back to a Point.
        let deserialized: AuthAccountResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}