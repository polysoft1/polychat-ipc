use tokio;
extern crate polychat_ipc;
use polychat_ipc::polychat_plugin_sdk_rust::entrypoint;
//use log::info;

#[tokio::main]
async fn main() {
    println!("Test Example plugin starting.");
    entrypoint::run_plugin().await;
    println!("Test Example plugin finished running.");
}