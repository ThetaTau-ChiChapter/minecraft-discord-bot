use tokio::net::TcpStream;
use tokio::sync::Mutex;
use rcon::Connection;

use crate::Error;

type RconConnection = Connection<TcpStream>;
pub struct State {
    rcon: Mutex<RconConnection>,
    rcon_address: String,
    rcon_password: String,
}

impl State {
    /// Create a new State instance, connecting to the RCON server
    pub async fn new(rcon_address: String, rcon_password: String) -> Self {
        let rcon = Connection::<TcpStream>::connect(&rcon_address, &rcon_password)
            .await
            .expect("Failed to connect to RCON server");
        Self {
            rcon: Mutex::new(rcon),
            rcon_address,
            rcon_password,
        }
    }

    /// Run an RCON command, transparently reconnecting once if the server
    /// closed the connection (e.g. after sitting idle) before retrying.
    pub async fn rcon_cmd(&self, command: &str) -> Result<String, Error> {
        let mut rcon = self.rcon.lock().await;

        match rcon.cmd(command).await {
            Err(rcon::Error::Io(_)) => {
                *rcon =
                    Connection::<TcpStream>::connect(&self.rcon_address, &self.rcon_password)
                        .await?;
                Ok(rcon.cmd(command).await?)
            }
            result => Ok(result?),
        }
    }
}