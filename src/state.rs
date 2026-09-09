use std::cmp::min;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::Mutex;
use rcon::Connection;
use sea_orm::DatabaseConnection;

use crate::Error;
use crate::config::Config;
use crate::migrations::Migrator;

type RconConnection = Connection<TcpStream>;
pub struct State {
    pub db: DatabaseConnection,
    rcon: Mutex<RconConnection>,
    rcon_address: String,
    rcon_password: String,
}

impl State {
    /// Create a new State instance, connecting to the RCON server and the database
    pub async fn new(
        config: Config,
    ) -> Self {
        let rcon = Connection::<TcpStream>::connect(config.minecraft.rcon_address.clone(), &config.minecraft.rcon_password)
            .await
            .expect("Failed to connect to RCON server");

        let db = create_database_connection(
            config.database.url,
            config.database.schema,
            config.database.pool_size,
        )
        .await
        .expect("Failed to connect to the database");

        Self {
            db,
            rcon: Mutex::new(rcon),
            rcon_address: config.minecraft.rcon_address,
            rcon_password: config.minecraft.rcon_password,
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

/// Create a new Postgresql connection
///
/// This method creates a new Postgresql connection and runs any migrations, if needed.
///
/// # Arguments
///
/// * `url` - The Postgres connection string
/// * `schema` - The Postgres schema. Defaults to `public`
/// * `pool_size` - The number of connections in the pool. Defaults to `3`
///
///  # Returns
///
///    A new [DatabaseConnection] object
///
pub async fn create_database_connection(
    url: String,
    schema: Option<String>,
    pool_size: Option<u32>,
) -> Result<DatabaseConnection, Error> {
    use sea_orm::{ConnectOptions, ConnectionTrait, Database};
    use sea_orm_migration::MigratorTrait;

    let mut connection_options = ConnectOptions::new(url);
    connection_options
        .max_connections(pool_size.unwrap_or(3))
        .min_connections(min(2, pool_size.unwrap_or(2)))
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(15))
        .max_lifetime(Duration::from_secs(15))
        .sqlx_logging(false);

    let schema = schema.clone().unwrap_or_else(|| "rewards".to_string());

    // if a schema is specified, set the search path to it (and public as fallback)
    if schema != "public" {
        connection_options.set_schema_search_path(format!("{},public", schema));
    }

    // connection
    tracing::trace!("Creating database connection...");
    let db = Database::connect(connection_options).await?;

    // run migrations if needed
    tracing::debug!("Running migrations...");
    if schema != "public" {
        db.execute_unprepared(&format!("CREATE SCHEMA IF NOT EXISTS {schema}"))
            .await?;
    }
    Migrator::up(&db, None).await?;

    Ok(db)
}
