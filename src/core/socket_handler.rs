use crate::{
    api::schema::{
        instructions::{SerializablePluginInstr, DeserializableCoreInstr},
    },
    utils::socket::*
};

use log::{debug, error, warn, trace};
use interprocess::local_socket::{
    NameTypeSupport, 
    tokio::{LocalSocketListener, LocalSocketStream, OwnedReadHalf, OwnedWriteHalf}
};
use serde::Serialize;
use std::{path::Path, fs, fmt::Debug};

use anyhow::Result;

#[derive(Debug)]
pub struct SocketHandler {
    socket_name: String,
    listener: LocalSocketListener,
    read: Option<OwnedReadHalf>,
    write: Option<OwnedWriteHalf>
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
     * A [Result] is returned, if successful the SocketHandler is provided
     * otherwise, an [Error](std::error::Error) is returned containing the error.
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
            socket_name: name,
            read: None,
            write: None
        })
    }

    /**
     * Reads an instruction from the socket connection, fails when an unrecongized
     * instruction is received
     * 
     * # Return
     * Upon a successful read and parse from the socket, a [CoreInstruction]
     * is returned.  Otherwise an [Error](std::error::Error) is returned.
     **/
    pub async fn get_instruction(&mut self) -> Result<DeserializableCoreInstr> {
        self.update_owned_split().await?;
        let data = self.get_data().await?;
        debug!("Converting");
        convert_str_to_struct::<DeserializableCoreInstr>(&data)
    }

    /**
     * If the read or write part of the connection are not associated with this object, a new connection is
     * gathered, and then the read/write parts are assigned to this object
     * 
     * # Returns
     * A [Result] is returned, void if successful, [Error](std::error::Error) if unsuccessful
     **/
    async fn update_owned_split(&mut self) -> Result<()> {
        trace!("Checking if read/write needs updating");
        if self.read.is_none() || self.write.is_none() {
            debug!("Updating read/write associations");
            let (read, write) = self.get_connection().await?.into_split();
            self.read = Some(read);
            self.write = Some(write);
        }
        Ok(())
    }

    /**
     * Gets a connection from the socket
     * 
     * # Returns
     * A [Result] is returned, with [LocalSocketStream] on success, and [Error](std::error::Error) on failure
     **/
    async fn get_connection(&self) -> Result<LocalSocketStream> {
        debug!("Fetching new connection");
        match self.listener.accept().await {
            Ok(c) => {
                debug!("Found new connection");
                Ok(c)
            },
            Err(e) => {
                warn!("Could not accept a socket connection: {}", e);
                return Err(e.into());
            }
        }
    }
    /**
     * Sends a [PluginInstruction] over the socket for the plugin process to handle.
     * 
     * # Parameters
     * - inst ([PluginInstruction]): The instruction to be sent
     * 
     * # Returns
     * A [Result], void on success, [Error](std::error::Error) on failure
     **/
    pub async fn send_plugin_instruction<P: Serialize + Debug>(&mut self, inst: &SerializablePluginInstr<P>) -> Result<()> {
        self.update_owned_split().await?;
        let payload = match convert_struct_to_str(inst) {
            Ok(s) => s,
            Err(e) => {
                warn!("Could not convert instruction to a String!");
                return Err(e.into());
            }
        };
        
        return send_str_over_ipc(&payload, self.write.as_mut().unwrap()).await;
    }

    /**
     * Reads data from the socket connection, and returns what it found
     * 
     * # Returns
     * A [Result] containing the received data in a [String] or a [Error](std::error::Error) on failure
     **/
    async fn get_data(&mut self) -> Result<String> {
        self.update_owned_split().await?;
        debug!("Fetching data from socket");
        let reader = self.read.as_mut().unwrap();
        return receive_line(reader).await;
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
    use claims::assert_ok;

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
}
