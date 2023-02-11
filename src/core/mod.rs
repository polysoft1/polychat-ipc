pub mod socket_handler;

use socket_handler::SocketHandler;

use anyhow::Result;

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
    pub fn new() -> Result<Core> {
        let handler = SocketHandler::new("polychat")?;

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
