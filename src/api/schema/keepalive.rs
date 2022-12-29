use serde::{Serialize, Deserialize};

/// A simple instruction for keepalive pings
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct KeepaliveInstruction {
    pub id: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;
    use log::debug;

    // Serialization + Deserialization tests
    // For all of the types, these tests serialize and deserialize them to
    // ensure it behaves as expected
    // To see the serialized structs as json when you run the tests, run it
    // as `cargo test -- --nocapture`
    #[test]
    fn test_keepalive_instruction_serialization() {
        let original = KeepaliveInstruction { id: 0 };
        let serialized = serde_json::to_string(&original).unwrap();

        debug!("serialized keepalive = {}", serialized);

        // Convert the JSON string back to a Point.
        let deserialized: KeepaliveInstruction = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}