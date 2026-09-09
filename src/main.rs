mod bot;
mod config;
mod entities;
mod migrations;
mod state;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let path = std::env::current_dir()?;

    let conf = config::Config::parse("./config")?;

    bot::start_bot(conf).await
}
