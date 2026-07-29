use poise::serenity_prelude as serenity;
use rcon::Connection;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

type RconConnection = Connection<TcpStream>;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Data {
    rcon: Mutex<RconConnection>,
}

/// Whitelist a player on the Minecraft server
#[poise::command(slash_command)]
async fn whitelist(
    ctx: Context<'_>,
    #[description = "Player name to whitelist"] player: String,
) -> Result<(), Error> {
    let response = {
        let mut rcon = ctx.data().rcon.lock().await;
        rcon.cmd(&format!("whitelist add {player}")).await?
    };

    ctx.say(format!("```\n{response}\n```")).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();

    let discord_token =
        std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN in environment");
    let rcon_address = std::env::var("RCON_ADDRESS").expect("Missing RCON_ADDRESS in environment");
    let rcon_password =
        std::env::var("RCON_PASSWORD").expect("Missing RCON_PASSWORD in environment");

    let rcon = Connection::<TcpStream>::connect(&rcon_address, &rcon_password)
        .await
        .expect("Failed to connect to RCON server");

    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![whitelist()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    rcon: Mutex::new(rcon),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(discord_token, intents)
        .framework(framework)
        .await;

    client?.start().await?;

    Ok(())
}
