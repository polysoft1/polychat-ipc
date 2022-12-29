use log::{debug, error, warn};

use tokio::sync::oneshot::Sender;
use interprocess::local_socket::{
    NameTypeSupport, 
    tokio::{LocalSocketListener, LocalSocketStream}
};
use futures::{
    io::BufReader, AsyncBufReadExt
};

pub struct SocketHandler {
    listener: LocalSocketListener
}


impl SocketHandler {
    /// Creates a new SocketHandler and listens either on a namespaced local socket
    /// or a file path socket.
    /// 
    /// # Arguments
    /// ## `socket_name`
    /// The name to be used for the filepath or namespace
    /// 
    /// # Returns
    /// A `Result` is returned, if successful the SocketHandler is provided
    /// otherwise, a `String` is returned containing the error.
    /// 
    /// # Platform-Dependent Behavior
    /// * Windows/Linux - Creates a namespaced socket (@[`socket_name`](#socket_name).sock)
    /// * BSD/Mac/*NIX - Creates a filepath socket at /tmp/[`socket_name`](#socket_name).sock
    pub fn new(socket_name: &str) -> Result<SocketHandler, String> {
        let name = {
            use NameTypeSupport::*;
            match NameTypeSupport::query() {
                OnlyPaths => format!("/tmp/{}.sock", socket_name),
                OnlyNamespaced | Both => format!("@{}.sock", socket_name),
            }
        };

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
            listener
        })
    }

    ///
    pub async fn run(&self, notify: Sender<()>) {
        let _ = notify.send(());
        

        loop {
            let conn = match self.listener.accept().await {
                Ok(c) => c,
                Err(e) => {
                    warn!("Could not accept a socket connection: {}", e);
                    continue;
                }
            };

            self.handle_connection(conn).await;
        }
    }
    async fn handle_connection(&self, conn: LocalSocketStream) {
        let (reader, mut writer) = conn.into_split();
        let mut reader = BufReader::new(reader);
        let mut data = String::with_capacity(128);

        let read_res = reader.read_line(&mut data).await;
        
        match read_res {
            Ok(size) => {
                debug!("Received {} bytes from a client", size);
                debug!("Message contents: {}", data);
            },
            Err(e) => {
                warn!("Could not read from client: {}", e);
                return;
            }
        }
    }
}