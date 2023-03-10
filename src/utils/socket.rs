use std::fmt::Debug;
use std::any::type_name;

use interprocess::local_socket::tokio::{OwnedReadHalf, OwnedWriteHalf};
use interprocess::local_socket::NameTypeSupport;
use futures::{
    io::BufReader, AsyncBufReadExt, AsyncWriteExt
};
use log::{debug, warn, trace};
use serde::{Deserialize, Serialize};

use anyhow::Result;

pub async fn receive_line(reader: &mut OwnedReadHalf) -> Result<String> {
    let mut bufreader = BufReader::new(reader);
    let mut data = String::with_capacity(128);

    match bufreader.read_line(&mut data).await {
        Ok(size) => {
            debug!("Received {} bytes from connection", size);
        },
        Err(e) => {
            warn!("Could not read line from connection: {}", e.to_string());
            return Err(e.into());
        }
    };

    Ok(data)
}

pub fn convert_str_to_struct<'a, T>(data: &'a String) -> Result<T> where T: Deserialize<'a>{
    let template_type_name = type_name::<T>();
    trace!("Attempting to deserialize '{}' into {}", data, template_type_name);
    match serde_json::from_str::<T>(data) {
        Ok(s_struct) => Ok(s_struct),
        Err(e) => {
            warn!("Error serializing data into {}: {}", template_type_name, e.to_string());
            Err(e.into())
        }
    }
}

pub fn convert_struct_to_str<'a, T: 'a>(msg: &T) -> Result<String> where T: Serialize + Debug {
    match serde_json::to_string(msg) {
        Ok(s) => Ok(s),
        Err(e) => {
            debug!("Error serializing {:?}: {}", msg, e.to_string());
            return Err(e.into());
        }
    }
}

pub async fn send_str_over_ipc(msg: &String, ipc: &mut OwnedWriteHalf) -> Result<()> {
    let payload = format!("{}\n", msg);
    trace!("Sending {} across", msg);
    match ipc.write_all(payload.as_bytes()).await {
        Ok(_) => {
            debug!("Data sent");
            Ok(())
        },
        Err(e) => {
            warn!("Error sending data: {}", e.to_string());
            Err(e.into())
        }
    }
}

pub fn get_socket_name<S>(name: S) -> String where S: Into<String> + std::fmt::Display {
    match NameTypeSupport::query() {
        NameTypeSupport::OnlyPaths | NameTypeSupport::Both => format!("/tmp/{}.sock", name),
        NameTypeSupport::OnlyNamespaced => format!("@{}.sock", name)
    }
}
