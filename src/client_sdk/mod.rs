use futures::{AsyncWriteExt, AsyncReadExt};
use interprocess::local_socket::{
    tokio::{
        LocalSocketStream, OwnedReadHalf, OwnedWriteHalf,
    },
    NameTypeSupport
};

use crate::api::schema::instructions::{
    CoreInstruction, PluginInstruction
};


pub struct SocketCommunicator {
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf
}

impl SocketCommunicator {
    pub async fn new() -> Result<SocketCommunicator, String> {
        let stream = match LocalSocketStream::connect(get_socket_name()).await {
            Ok(s) => s,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        let (reader, writer) = stream.into_split();
        Ok(SocketCommunicator { 
            reader,
            writer
        })
    }
    pub async fn send_core_message(&mut self, msg: CoreInstruction) -> Result<(), String>{
        let data = match serde_json::to_string(&msg) {
            Ok(s) => s,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        match self.writer.write_all(data.as_bytes()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string())
        }
    }

    pub async fn recv_instruction(&mut self) -> Result<PluginInstruction, String> {
        let mut buffer = String::with_capacity(128);

        match self.reader.read_to_string(&mut buffer).await {
            Ok(_) => {},
            Err(e) => {
                return Err(e.to_string());
            }
        }
        
        match serde_json::from_str::<PluginInstruction>(&buffer) {
            Ok(ins) => Ok(ins),
            Err(e) => Err(e.to_string())
        }
    }
}

fn get_socket_name() -> &'static str {
    use NameTypeSupport::*;

    match NameTypeSupport::query() {
        OnlyPaths | Both => "/tmp/polychat.sock",
        OnlyNamespaced => "@polychat.sock"
    }
}