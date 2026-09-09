use dbutton_macro::{create_dbutton, dbutton};
use poise::serenity_prelude::{
    ButtonStyle, Colour, ComponentInteraction, CreateActionRow, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use super::utils::not_registered_embed;
use crate::{
    Error,
    bot::{Context, utils::defer_response},
    entities::users,
    state::State,
};

/// Cost, in game cash, of smiting a player.
const SMITE_COST: u32 = 25;

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

/// Smite a player, charging them $SMITE_COST if it succeeds
///
/// Asks for confirmation before actually performing the smite.
#[poise::command(slash_command)]
pub async fn smite(
    ctx: Context<'_>,
    #[description = "Minecraft username to smite"] player: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    ctx.send(
        poise::CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("Confirm Smite")
                    .description(format!(
                        "Are you sure you want to smite `{player}`? This will charge them ${SMITE_COST} if it succeeds."
                    ))
                    .colour(Colour::ORANGE),
            )
            .components(vec![CreateActionRow::Buttons(vec![
                create_dbutton!(confirm_smite, player.clone())
                    .label("Smite")
                    .style(ButtonStyle::Danger),
                create_dbutton!(cancel_smite, player).label("Cancel"),
            ])]),
    )
    .await?;

    Ok(())
}

/// Button press to confirm a pending smite: checks the target can afford it, performs the
/// smite, and only then charges them.
#[dbutton]
pub async fn confirm_smite(
    ctx: &poise::serenity_prelude::Context,
    interaction: &ComponentInteraction,
    _framework: poise::FrameworkContext<'_, State, Error>,
    state: &State,
    player: String,
) -> Result<(), Error> {
    defer_response(ctx, interaction).await?;

    let embed = perform_smite(state, &player).await?;

    interaction
        .edit_response(
            ctx.http.clone(),
            EditInteractionResponse::new().embed(embed).components(vec![]),
        )
        .await?;

    Ok(())
}

/// Button press to cancel a pending smite.
#[dbutton]
pub async fn cancel_smite(
    ctx: &poise::serenity_prelude::Context,
    interaction: &ComponentInteraction,
    _framework: poise::FrameworkContext<'_, State, Error>,
    _state: &State,
    player: String,
) -> Result<(), Error> {
    interaction
        .create_response(
            ctx.http.clone(),
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(
                        CreateEmbed::new()
                            .title("Smite Cancelled")
                            .description(format!("Smiting `{player}` was cancelled."))
                            .colour(Colour::LIGHT_GREY),
                    )
                    .components(vec![]),
            ),
        )
        .await?;

    Ok(())
}

/// Check `player` can afford [`SMITE_COST`], smite them, and - only if that succeeds - charge
/// them. Returns the result embed to show the user.
async fn perform_smite(state: &State, player: &str) -> Result<CreateEmbed, Error> {
    let balance_response = state.rcon_cmd(&format!("balance {player}")).await?;
    let Some(balance) = parse_dollar_amount(&balance_response) else {
        return Ok(smite_error_embed("Error: Player not found."));
    };

    if balance < SMITE_COST as f64 {
        return Ok(smite_error_embed(&format!(
            "`{player}` can't afford to be smitten - they only have ${balance:.2} and smiting costs ${SMITE_COST}."
        )));
    }

    let smite_response = state.rcon_cmd(&format!("smite {player}")).await?;
    let smite_response = smite_response.trim();

    if smite_response.starts_with("Error") {
        return Ok(smite_error_embed(smite_response));
    }

    state
        .rcon_cmd(&format!("eco take {player} {SMITE_COST}"))
        .await?;

    Ok(CreateEmbed::new()
        .title("Smitten!")
        .description(smite_response)
        .field("Cost", format!("${SMITE_COST}"), true)
        .colour(Colour::DARK_PURPLE))
}

/// Build a "smite failed" error embed with the given `message`.
fn smite_error_embed(message: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title("Smite Failed")
        .description(message)
        .colour(Colour::RED)
}

/// Parse an economy RCON response such as `Balance of Steve: $50` and return the numeric dollar
/// amount.
fn parse_dollar_amount(response: &str) -> Option<f64> {
    let (_, amount) = response.rsplit_once('$')?;
    amount.trim().replace(',', "").parse().ok()
}
