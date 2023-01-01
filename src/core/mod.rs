pub mod socket_handler;

use socket_handler::SocketHandler;

pub struct Core {
    socket_hadler: SocketHandler
}

impl Core {
    pub fn new() -> Result<Core, String> {
        let handler = match SocketHandler::new("polychat") {
            Ok(s) => s,
            Err(e) => {
                return Err(e);
            }
        };

        Ok(Core {
            socket_hadler: handler
        })
    }

    pub async fn run(&self) {
        self.socket_hadler.run().await;
    }
}
