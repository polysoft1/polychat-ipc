use std::fmt::Debug;

use interprocess::local_socket::{
    tokio::{
        LocalSocketStream, OwnedReadHalf, OwnedWriteHalf,
    }
};
use serde::Serialize;

use crate::{
    api::schema::instructions::{SerializableCoreInstr, DeserializablePluginInstr},
    utils::socket::*
};

use anyhow::Result;

#[derive(Debug)]
pub struct IPCClient {
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf
}

/// The component that handles connecting to the IPC socket or pipe, as well as
/// serializing and deserializing the instructions sent each way.
impl IPCClient {
    pub async fn new(name: &String) -> Result<IPCClient> {
        let stream = match LocalSocketStream::connect(get_socket_name(name)).await {
            Ok(s) => s,
            Err(e) => {
                return Err(e.into());
            }
        };
        let (reader, writer) = stream.into_split();
        Ok(IPCClient { 
            reader,
            writer
        })
    }

    pub async fn send_core_instruction<P: Serialize + Debug>(&mut self, msg: &SerializableCoreInstr<P>) -> Result<()>{
        let payload = convert_struct_to_str(&msg)?;
        Ok(send_str_over_ipc(&payload, &mut self.writer).await?)
    }

    pub async fn recv_plugin_instruction(&mut self) -> Result<DeserializablePluginInstr> {
        let data  = match receive_line(&mut self.reader).await {
            Ok(s) => s,
            Err(e) => {
                return Err(e.into());
            }
        };
        
        return match convert_str_to_struct::<DeserializablePluginInstr>(&data) {
            Ok(plugin_instr) => {
                Ok(plugin_instr)
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}
