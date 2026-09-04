use poise::serenity_prelude as serenity;

use crate::{Error, state::State};

mod commands;

type Context<'a> = poise::Context<'a, State, Error>;


pub async fn start_bot(
    discord_token: String,
    rcon_address: String,
    rcon_password: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::whitelist()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(State::new(rcon_address, rcon_password).await)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(discord_token, intents)
        .framework(framework)
        .await;

    client?.start().await?;

    Ok(())
}