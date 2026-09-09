use poise::serenity_prelude::{Colour, CreateEmbed};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use super::utils::not_registered_embed;
use crate::{Error, bot::Context, entities::users};

/// Check your linked Minecraft account's balance
#[poise::command(slash_command)]
pub async fn balance(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let discord_id = ctx.author().id.get() as i64;
    let user = users::Entity::find()
        .filter(users::Column::DiscordId.eq(discord_id))
        .one(&ctx.data().db)
        .await?;

    let Some(minecraft_username) = user.and_then(|user| user.minecraft_username) else {
        ctx.send(poise::CreateReply::default().embed(not_registered_embed()).ephemeral(true))
            .await?;
        return Ok(());
    };

    let response = ctx
        .data()
        .rcon_cmd(&format!("balance {minecraft_username}"))
        .await?;

    // Response is of the form `Balance of {username}: $50` - pull out just the `$50` part.
    let amount = response
        .rsplit_once('$')
        .map(|(_, amount)| format!("${}", amount.trim()))
        .unwrap_or_else(|| response.trim().to_owned());

    ctx.send(
        poise::CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("Balance")
                    .description(format!("Balance for `{minecraft_username}`"))
                    .field("Balance", amount, false)
                    .colour(Colour::GOLD),
            )
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
