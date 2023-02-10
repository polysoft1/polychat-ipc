use interprocess::local_socket::{
    tokio::{
        LocalSocketStream, OwnedReadHalf, OwnedWriteHalf,
    }
};

use crate::{
    api::schema::instructions::{
        CoreInstruction, PluginInstruction
    },
    utils::socket::*
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

    pub async fn send_core_instruction(&mut self, msg: &CoreInstruction) -> Result<(), String>{
        let payload = convert_struct_to_str(msg)?;
        Ok(send_str_over_ipc(&payload, &mut self.writer).await?)
    }

    pub async fn recv_plugin_instruction(&mut self) -> Result<PluginInstruction, String> {
        let data  = match receive_line(&mut self.reader).await {
            Ok(s) => s,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        
        convert_str_to_struct::<PluginInstruction>(&data)
    }
}
