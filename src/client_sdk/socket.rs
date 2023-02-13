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

use anyhow::Result;

#[derive(Debug)]
pub struct SocketCommunicator {
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf
}

impl SocketCommunicator {
    pub async fn new(name: String) -> Result<SocketCommunicator> {
        let stream = match LocalSocketStream::connect(get_socket_name(name)).await {
            Ok(s) => s,
            Err(e) => {
                return Err(e.into());
            }
        };
        let (reader, writer) = stream.into_split();
        Ok(SocketCommunicator { 
            reader,
            writer
        })
    }

    pub async fn send_core_instruction(&mut self, msg: &CoreInstruction) -> Result<()>{
        let payload = convert_struct_to_str(msg)?;
        Ok(send_str_over_ipc(&payload, &mut self.writer).await?)
    }

    pub async fn recv_plugin_instruction(&mut self) -> Result<PluginInstruction> {
        let data  = match receive_line(&mut self.reader).await {
            Ok(s) => s,
            Err(e) => {
                return Err(e.into());
            }
        };
        
        convert_str_to_struct::<PluginInstruction>(&data)
    }
}
