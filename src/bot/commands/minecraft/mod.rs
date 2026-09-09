pub mod economy;
mod utils;

pub use economy::balance;

use dbutton_macro::{create_dbutton, dbutton};
use poise::{serenity_prelude::{
    ActionRowComponent, Colour, ComponentInteraction, CreateActionRow, CreateEmbed,
    CreateInputText, CreateInteractionResponse, CreateInteractionResponseMessage, CreateModal,
    InputTextStyle, ModalInteraction,
}};
use rand::RngExt;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::{
    Error,
    bot::{Context, utils::defer_response},
    entities::{pending_user_codes, users},
    state::State,
};

const REGISTRATION_CODE_LENGTH: usize = 4;
const REGISTRATION_CODE_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// Custom id of the modal opened by [`enter_code`]. Also used by the event handler to
/// recognize the modal submission and route it to [`handle_enter_code_modal_submit`].
pub(crate) const ENTER_CODE_MODAL_ID: &str = "enter_code_modal";
/// Custom id of the code input field inside the [`ENTER_CODE_MODAL_ID`] modal.
const ENTER_CODE_INPUT_ID: &str = "code";

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


/// Register a player with their Minecraft account
#[poise::command(slash_command)]
pub async fn register_minecraft(
    ctx: Context<'_>,
    #[description = "Your minecraft username"] player: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    generate_store_and_send_registration_code(ctx.data(), &player).await?;

    ctx.send(
        poise::CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("Minecraft Registration")
                    .description(format!(
                        "A registration code has been messaged in game to the Minecraft account `{player}`."
                    ))
                    .field(
                        "Next steps",
                        "Enter the code below to link your Discord account to that Minecraft account.",
                        false,
                    )
                    .field(
                        "Haven't joined the server yet?",
                        "Join first, then press **Send New Code** to get a new code messaged to you in game.",
                        false,
                    )
                    .colour(Colour::DARK_GREEN),
            )
            .components(vec![CreateActionRow::Buttons(vec![
                create_dbutton!(send_new_code, player).label("Send New Code"),
                create_dbutton!(enter_code).label("Enter Code"),
            ])]),
    )
    .await?;

    Ok(())
}

/// Button press to send a new code to the user
#[dbutton]
pub async fn send_new_code(
    ctx: &poise::serenity_prelude::Context,
    interaction: &ComponentInteraction,
    _framework: poise::FrameworkContext<'_, State, Error>,
    state: &State,
    player: String,
) -> Result<(), Error> {
    defer_response(ctx, interaction).await?;

    generate_store_and_send_registration_code(state, &player).await?;
    Ok(())
}

/// Button press to open the "enter code" modal
///
/// Note: unlike other button handlers, this must respond with the modal directly rather than
/// calling [`defer_response`] first - Discord only allows a modal to be shown as an
/// interaction's first response.
#[dbutton]
pub async fn enter_code(
    ctx: &poise::serenity_prelude::Context,
    interaction: &ComponentInteraction,
    _framework: poise::FrameworkContext<'_, State, Error>,
    _state: &State,
) -> Result<(), Error> {
    let modal = CreateModal::new(ENTER_CODE_MODAL_ID, "Enter your registration code").components(
        vec![CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, "Code", ENTER_CODE_INPUT_ID)
                .placeholder("XXXX")
                .min_length(REGISTRATION_CODE_LENGTH as u16)
                .max_length(REGISTRATION_CODE_LENGTH as u16),
        )],
    );

    interaction
        .create_response(ctx.http.clone(), CreateInteractionResponse::Modal(modal))
        .await?;

    Ok(())
}

/// Handle submission of the [`ENTER_CODE_MODAL_ID`] modal.
///
/// Validates the submitted code against `pending_user_codes` and, if valid, links the
/// submitting Discord user to the associated Minecraft username in `users` (updating the
/// row if the user already registered one before), then consumes the pending code.
pub(crate) async fn handle_enter_code_modal_submit(
    ctx: &poise::serenity_prelude::Context,
    modal: &ModalInteraction,
    state: &State,
) -> Result<(), Error> {
    let submitted_code = modal
        .data
        .components
        .iter()
        .flat_map(|row| &row.components)
        .find_map(|component| match component {
            ActionRowComponent::InputText(input) if input.custom_id == ENTER_CODE_INPUT_ID => {
                input.value.clone()
            }
            _ => None,
        })
        .unwrap_or_default()
        .trim()
        .to_uppercase();

    let pending = pending_user_codes::Entity::find()
        .filter(pending_user_codes::Column::Code.eq(submitted_code.as_str()))
        .one(&state.db)
        .await?;

    let Some(pending) = pending else {
        modal
            .create_response(
                ctx.http.clone(),
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().ephemeral(true).content(
                        "That code is invalid or has expired. Run `/register_minecraft` again to get a new one.",
                    ),
                ),
            )
            .await?;
        return Ok(());
    };

    let discord_id = modal.user.id.get() as i64;
    let existing_user = users::Entity::find()
        .filter(users::Column::DiscordId.eq(discord_id))
        .one(&state.db)
        .await?;

    match existing_user {
        Some(user_model) => {
            let mut active: users::ActiveModel = user_model.into();
            active.minecraft_username = Set(Some(pending.minecraft_username.clone()));
            active.update(&state.db).await?;
        }
        None => {
            users::ActiveModel {
                id: Set(Uuid::new_v4()),
                discord_id: Set(discord_id),
                minecraft_username: Set(Some(pending.minecraft_username.clone())),
            }
            .insert(&state.db)
            .await?;
        }
    }

    pending_user_codes::Entity::delete_by_id(pending.id)
        .exec(&state.db)
        .await?;

    modal
        .create_response(
            ctx.http.clone(),
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true).content(
                format!(
                    "You're registered! Your Discord account is now linked to Minecraft username `{}`.",
                    pending.minecraft_username
                ),
            )),
        )
        .await?;

    Ok(())
}

/// Generate a random 4-character, upper-case alphanumeric code.
fn random_code() -> String {
    let mut rng = rand::rng();
    (0..REGISTRATION_CODE_LENGTH)
        .map(|_| REGISTRATION_CODE_CHARSET[rng.random_range(0..REGISTRATION_CODE_CHARSET.len())] as char)
        .collect()
}

/// Keep generating random codes until one that isn't already in `pending_user_codes` is found.
async fn generate_unique_code(db: &DatabaseConnection) -> Result<String, Error> {
    loop {
        let code = random_code();
        let exists = pending_user_codes::Entity::find()
            .filter(pending_user_codes::Column::Code.eq(code.as_str()))
            .one(db)
            .await?;

        if exists.is_none() {
            return Ok(code);
        }
    }
}

/// Generate a unique registration code for `minecraft_username` and store it in
/// `pending_user_codes`, updating the existing row if the user already has one pending.
///
/// Returns the generated code.
async fn generate_store_and_send_registration_code(
    state: &State,
    minecraft_username: &str,
) -> Result<(), Error> {
    let code = generate_unique_code(&state.db).await?;

    let existing = pending_user_codes::Entity::find()
        .filter(pending_user_codes::Column::MinecraftUsername.eq(minecraft_username))
        .one(&state.db)
        .await?;

    match existing {
        Some(model) => {
            let mut active: pending_user_codes::ActiveModel = model.into();
            active.code = Set(code.clone());
            active.update(&state.db).await?;
        }
        None => {
            pending_user_codes::ActiveModel {
                id: Set(Uuid::new_v4()),
                code: Set(code.clone()),
                minecraft_username: Set(minecraft_username.to_owned()),
            }
            .insert(&state.db)
            .await?;
        }
    }

    Ok(())
}