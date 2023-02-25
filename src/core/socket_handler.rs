use crate::{
    api::schema::{
        instructions::{CoreInstructionType, SerializablePluginInstr, DeserializableCoreInstr},
    },
    utils::socket::*
};

use log::{debug, error, warn, trace};
use interprocess::local_socket::{
    NameTypeSupport, 
    tokio::{LocalSocketListener, LocalSocketStream}
};
use serde::Serialize;
use std::{path::Path, fs, fmt::Debug};

use anyhow::Result;

#[derive(Debug)]
pub struct SocketHandler {
    socket_name: String,
    listener: LocalSocketListener
}


impl SocketHandler {
    /** Creates a new SocketHandler and listens either on a namespaced local socket
     * or a file path socket.
     * 
     * # Arguments
     * ## `socket_name`
     * The name to be used for the filepath or namespace
     * 
     * # Returns
     * A `Result` is returned, if successful the SocketHandler is provided
     * otherwise, a `String` is returned containing the error.
     * 
     * # Platform-Dependent Behavior
     * - Windows/Linux - Creates a namespaced socket (@[`socket_name`](#socket_name).sock)
     * - BSD/Mac/\*NIX - Creates a filepath socket at /tmp/[`socket_name`](#socket_name).sock
     **/
    pub fn new<S>(socket_name: S) -> Result<Self> where S: Into<String> + std::fmt::Display {
        let name = get_socket_name(socket_name);

        debug!("Attempting to start server at {}", name);
        let listener = match LocalSocketListener::bind(name.clone()) {
            Ok(l) => l,
            Err(e) => {
                error!("Could not start server: {}", e);
                return Err(e.into());
            }
        };
        debug!("Server started at {}", name);
        Ok(SocketHandler{
            listener,
            socket_name: name
        })
    }

    /** The main thread loop of SocketHandler, this will handle the
     * incoming socket connections and direct them off to where they need to go
     * 
     * **THIS FUNCTION CONTAINS AN INFINITE LOOP, RUN IT IN ITS OWN THREAD**
     */
    pub async fn run(&self) {      
        loop {
            let conn = match self.get_connection().await {
                Ok(c) => c,
                Err(_) => {
                    continue;
                }
            };

            let (mut reader, _) = conn.into_split();
            let data = match receive_line(&mut reader).await {
                Err(_) => {
                    continue;
                },
                Ok(s) => s
            };
            
            let _ = self.handle_recv_core_message(data).await;
        }
    }

    pub async fn get_connection(&self) -> Result<LocalSocketStream> {
        match self.listener.accept().await {
            Ok(c) => Ok(c),
            Err(e) => {
                warn!("Could not accept a socket connection: {}", e);
                return Err(e.into());
            }
        }
    }

    pub async fn send_plugin_instruction<P: Serialize + Debug>(&self, conn: LocalSocketStream, inst: &SerializablePluginInstr<P>) -> Result<()> {
        let (_, mut writer) = conn.into_split();
        let payload = match convert_struct_to_str(&inst) {
            Ok(s) => s,
            Err(e) => {
                warn!("Could not convert instruction to a String!");
                return Err(e.into());
            }
        };
        
        return send_str_over_ipc(&payload, &mut writer).await;
    }

    pub async fn get_core_instruction_data(&self, conn: LocalSocketStream) -> Result<String> {
        let (mut reader, _) = conn.into_split();
        return receive_line(&mut reader).await;
    }

    /**
     * Handles a message, serializing it to a [CoreInstruction] and then returning it
     * 
     * TEMPORARY FUNCTIONALITY: Log what kind of instruction was received
     * (this should be delegated off to whatever function handles the particular [CoreInstruction])
     * 
     * # Arguments
     * ## data
     * A [String] containing JSON data serializable to a [CoreInstruction]
     * 
     * # Returns
     * A [CoreInstruction] on success
     * 
     * A String containing error information on failure
     */
    pub async fn handle_recv_core_message(&self, data: String) -> Result<()> {
        // TODO: Add parameter for the trait for core instruction handler, and call the appropriate function.
        // Currently that function is call_core_handler
        trace!("Serializing {}", data);
        let data = match serde_json::from_str::<DeserializableCoreInstr>(data.as_str()) {
            Ok(data) => data,
            Err(e) => {
                debug!("Unrecognized instruction received");
                return Err(e.into());
            }
        };
        
        match data.instruction_type {
            CoreInstructionType::Init => {
                debug!("Init Instruction received");
            },
            CoreInstructionType::AuthAccountResponse => {
                debug!("Account Auth Instruction received");
            },
            CoreInstructionType::KeepaliveResponse => {
                debug!("Keep Alive Instruction received");
            }
        };

        Ok(())
    }
}

impl Drop for SocketHandler {
    fn drop(&mut self) {
        debug!("Attempting to close Socket {}", self.socket_name);
        use NameTypeSupport::*;
        match NameTypeSupport::query() {
            OnlyPaths | Both => {
                let path = Path::new(&self.socket_name);
                if path.exists() {
                    let res = fs::remove_file(path);
                    match res {
                        Ok(_) => {
                            debug!("Socket successfully removed");
                        },
                        Err(e) => {
                            error!("Could not clean up socket: {}", e.to_string());
                        }
                    }
                }
            },
            OnlyNamespaced => {},
        }
    }
}

#[cfg(test)]
mod test{
    use crate::core::SocketHandler;
    use rstest::*;
    use tokio_test::{assert_ok, assert_err};

    #[tokio::test]
    #[ignore = "Single Threaded test"]
    async fn create_socket_succeeds() {
        assert_ok!(SocketHandler::new("polychat"));
    }

    #[tokio::test]
    #[ignore = "Single Threaded test"]
    async fn socket_cleans_up_after_itself() {
        assert_ok!(SocketHandler::new("polychat"));
    }

    #[tokio::test]
    async fn socket_json_handles_malformed_instruction() {
        let socket = assert_ok!(SocketHandler::new("malformed_instruction"));
        let garbage = "{\"instruction_type\": \"Silliness\",\"payload\": {}}";

        assert_err!(socket.handle_recv_core_message(String::from(garbage)).await);
    }

    #[rstest]
    #[case("Init")]
    #[case("KeepaliveResponse")]
    #[case("AuthAccountResponse")]
    #[tokio::test]
    async fn socket_json_handles_valid_core_instruction_types(#[case] ins_type: String) {
        let socket = assert_ok!(SocketHandler::new(format!("{}_instruction", ins_type)));
        
        let inst = format!("{{\"instruction_type\": \"{}\", \"payload\": {{}} }}", ins_type);
        assert_ok!(socket.handle_recv_core_message(String::from(inst)).await);
    }

}
