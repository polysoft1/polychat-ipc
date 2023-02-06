pub mod socket_handler;

use socket_handler::SocketHandler;

pub struct Core {
    socket_hadler: SocketHandler
}

impl Core {
    /**
     * Creates a new Core object
     * 
     * # Returns
     * A valid `Core` object on success
     * A string describing the error on failure (more details can be found in logs, adjust `RUST_LOG` level)
     */
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

    /**
     * Starts the main loop of Core, allowing messages to be received from plugins
     * 
     * **THIS IS AN INFINITE LOOP**
     */
    pub async fn run(&self) {
        self.socket_hadler.run().await;
    }
}
