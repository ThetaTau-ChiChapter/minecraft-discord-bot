use crate::{Error, bot::Context};

/// Whitelist a player on the Minecraft server
#[poise::command(slash_command)]
pub async fn whitelist(
    ctx: Context<'_>,
    #[description = "Player name to whitelist"] player: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let response = ctx
        .data()
        .rcon_cmd(&format!("whitelist add {player}"))
        .await?;

    ctx.say(format!("```\n{response}\n```")).await?;
    Ok(())
}