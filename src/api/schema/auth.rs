use serde::{Serialize, Deserialize};

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


#[cfg(test)]
mod tests {
    use super::*;

    // Serialization + Deserialization tests
    // For all of the types, these tests serialize and deserialize them to
    // ensure it behaves as expected
    // To see the serialized structs as json when you run the tests, run it
    // as `cargo test -- --nocapture`
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