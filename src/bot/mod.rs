use poise::serenity_prelude as serenity;

use crate::{Error, config::Config, state::State};

mod commands;
mod event_handler;
mod utils;

type Context<'a> = poise::Context<'a, State, Error>;


/// Connect to discord and start the bot
pub async fn start_bot(
    config: Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let intents = serenity::GatewayIntents::non_privileged();

    let setup_config = config.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::minecraft::whitelist(),
                commands::minecraft::register_minecraft(),
                commands::bryce::bryce(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler::event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(State::new(setup_config).await)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(config.bot.discord_token, intents)
        .framework(framework)
        .await;

    client?.start().await?;

    Ok(())
}