mod bot;
mod state;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();

    let discord_token =
        std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN in environment");
    let rcon_address = std::env::var("RCON_ADDRESS").expect("Missing RCON_ADDRESS in environment");
    let rcon_password =
        std::env::var("RCON_PASSWORD").expect("Missing RCON_PASSWORD in environment");

    bot::start_bot(discord_token, rcon_address, rcon_password).await
}
