use futures::{
    io::BufReader, AsyncBufReadExt, AsyncWriteExt, AsyncReadExt, AsyncBufRead
};
use interprocess::local_socket::{
    tokio::{
        LocalSocketStream, OwnedReadHalf, OwnedWriteHalf,
    },
    NameTypeSupport
};
use log::debug;

use crate::api::schema::instructions::{
    CoreInstruction, PluginInstruction
};

#[derive(Debug)]
pub struct SocketCommunicator {
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf
}

impl SocketCommunicator {
    pub async fn new(name: String) -> Result<SocketCommunicator, String> {
        let stream = match LocalSocketStream::connect(get_socket_name(name)).await {
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

    pub async fn send_core_message(&mut self, msg: &CoreInstruction) -> Result<(), String>{
        let data = match serde_json::to_string(msg) {
            Ok(s) => s,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        debug!("Sending data to socket");
        let payload = format!("{}\n", data);
        match self.writer.write_all(payload.as_bytes()).await {
            Ok(_) => {
                debug!("Finished sending data to socket");
                debug!("Flushing writer");
                match self.writer.flush().await {
                    Err(e) => {
                        debug!("Could not flush buffer, {}", e.to_string());
                    }
                    Ok(_) => {
                        debug!("Successfully flushed buffer");
                    }
                };
                Ok(())
            },
            Err(e) => Err(e.to_string())
        }
    }

    pub async fn recv_instruction(&mut self) -> Result<PluginInstruction, String> {
        let mut reader = BufReader::new(&mut self.reader);
        let mut buffer = String::with_capacity(128);

        match reader.read_line(&mut buffer).await {
            Ok(_) => {},
            Err(e) => {
                debug!("Failed to read data from buffer! Received data {}, e: {}", buffer, e);
                return Err(e.to_string());
            }
        }
        
        match serde_json::from_str::<PluginInstruction>(&buffer) {
            Ok(ins) => Ok(ins),
            Err(e) => {
                debug!("Failed to deserialize PluginInstruction! Received data {}", buffer);
                Err(e.to_string())
            }
        }
    }
}

fn get_socket_name(name: String) -> String {
    use NameTypeSupport::*;

    match NameTypeSupport::query() {
        OnlyPaths | Both => format!("/tmp/{}.sock", name),
        OnlyNamespaced => format!("@{}.sock", name)
    }
}