use crate::api::schema::{
    instructions::{CoreInstruction, CoreInstructionType, PluginInstruction}
};

use log::{debug, error, warn, trace};
use interprocess::local_socket::{
    NameTypeSupport, 
    tokio::{LocalSocketListener, LocalSocketStream}
};
use futures::{
    io::BufReader, AsyncBufReadExt, AsyncWriteExt
};
use std::{path::Path, fs};

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
    pub fn new<S>(socket_name: S) -> Result<Self, String> where S: Into<String> + std::fmt::Display {
        let name = get_socket_name(socket_name);

        debug!("Attempting to start server at {}", name);
        let listener = match LocalSocketListener::bind(name.clone()) {
            Ok(l) => l,
            Err(e) => {
                error!("Could not start server: {}", e);
                return Err(e.to_string());
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
            let conn = match self.listener.accept().await {
                Ok(c) => c,
                Err(e) => {
                    warn!("Could not accept a socket connection: {}", e);
                    continue;
                }
            };

            let data = match self.recv_data(conn).await {
                None => {
                    continue;
                },
                Some(s) => s
            };
            
            let _ = self.handle_message(data).await;
        }
    }

    pub async fn get_connection(&self) -> Result<LocalSocketStream, String> {
        match self.listener.accept().await {
            Ok(c) => Ok(c),
            Err(e) => {
                warn!("Could not accept a socket connection: {}", e);
                return Err(e.to_string());
            }
        }
    }

    pub async fn send_plugin_instruction(&self, conn: LocalSocketStream, inst: &PluginInstruction) -> Result<(), String> {
        let (_, mut writer) = conn.into_split();
        let payload = match serde_json::to_string(&inst) {
            Ok(s) => s,
            Err(e) => {
                warn!("Could not convert instruction to a String!");
                return Err(e.to_string());
            }
        };
        let buffer = format!("{}\n", payload);
        match writer.write_all(buffer.as_bytes()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Could not write all data to buffer");
                Err(e.to_string())
            }
        }
    }

    pub async fn get_core_instruction_data(&self) -> Result<String, String> {
        let conn = match self.listener.accept().await {
            Ok(c) => c,
            Err(e) => {
                warn!("Could not accept a socket connection: {}", e);
                return Err(e.to_string());
            }
        };

        match self.recv_data(conn).await {
            None => {
                Ok("".to_string())
            },
            Some(s) => Ok(s)
        }
    }

    /** Receives data from a new connection, returning any data it might have sent
     * 
     * # Arguments
     * ## `conn`
     * A LocalSocketStream connection to a remote process
     * 
     * # Returns
     * `None` if no data was received (or the read errored out)
     * 
     * A `String` containing the data sent if the connection suceeded.
     */
    async fn recv_data(&self, conn: LocalSocketStream) -> Option<String> {
        let (reader, _) = conn.into_split();
        let mut reader = BufReader::new(reader);
        let mut data = String::with_capacity(128);

        let read_res = reader.read_line(&mut data).await;
        
        match read_res {
            Ok(size) => {
                debug!("Received {} bytes from a client", size);
                debug!("Message contents: {}", data);
                Some(data)
            },
            Err(e) => {
                warn!("Could not read from client: {}", e);
                return None;
            }
        }
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
    pub async fn handle_message(&self, data: String) -> Result<CoreInstruction, String> {
        trace!("Serializing {}", data);
        let data = match serde_json::from_str::<CoreInstruction>(data.as_str()) {
            Ok(data) => data,
            Err(e) => {
                debug!("Unrecognized instruction received");
                return Err(e.to_string());
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

        Ok(data)
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

fn get_socket_name<S>(name: S) -> String where S: Into<String> + std::fmt::Display {
    use NameTypeSupport::*;
    match NameTypeSupport::query() {
        OnlyPaths | Both => format!("/tmp/{}.sock", name),
        OnlyNamespaced => format!("@{}.sock", name),
    }
}

#[cfg(test)]
mod test{
    use crate::core::SocketHandler;
    use rstest::*;

    #[tokio::test]
    #[ignore = "Single Threaded test"]
    async fn create_socket_succeeds() {
        let socket = SocketHandler::new("polychat");

        assert!(socket.is_ok(), "Socket Handler was unable to init: {}", socket.unwrap_err());
    }

    #[tokio::test]
    #[ignore = "Single Threaded test"]
    async fn socket_cleans_up_after_itself() {
        let socket = SocketHandler::new("polychat");

        assert!(socket.is_ok(), "Second SocketHandler could not be initialized: {}", socket.unwrap_err());
    }

    #[tokio::test]
    async fn socket_json_handles_malformed_instruction() {
        let socket = SocketHandler::new("malformed_instruction");
        let garbage = "{\"instruction_type\": \"Silliness\",\"payload\": {}}";

        assert!(socket.is_ok(), "Could not init SocketHandler");
        let socket = socket.unwrap();

        let ins = socket.handle_message(String::from(garbage)).await;
        assert!(ins.is_err(), "SocketHandler did not err on malformed instruction");
    }

    #[rstest]
    #[case("Init")]
    #[case("KeepaliveResponse")]
    #[case("AuthAccountResponse")]
    #[tokio::test]
    async fn socket_json_handles_valid_core_instruction_types(#[case] ins_type: String) {
        let socket = SocketHandler::new(format!("{}_instruction", ins_type));
        assert!(socket.is_ok(), "Could not init SocketHandler");
        let socket = socket.unwrap();
        
        let inst = format!("{{\"instruction_type\": \"{}\", \"payload\": {{}} }}", ins_type);
        let result = socket.handle_message(String::from(inst)).await;
        assert!(result.is_ok(), "SocketHandler could not handle type: {}", ins_type);
    }

}
