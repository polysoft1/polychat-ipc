use serde::{Serialize, Deserialize};
use crate::api::schema::auth::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    // Serialization + Deserialization tests
    // For all of the types, these tests serialize and deserialize them to
    // ensure it behaves as expected
    // To see the serialized structs as json when you run the tests, run it
    // as `cargo test -- --nocapture`
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
}