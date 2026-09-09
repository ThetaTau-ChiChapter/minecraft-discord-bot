use poise::serenity_prelude::{Colour, CreateEmbed};

/// Build a standard "not registered" error embed.
///
/// Use this for any command that requires the invoking Discord user to have already linked a
/// Minecraft account via `/register_minecraft`.
pub fn not_registered_embed() -> CreateEmbed {
    CreateEmbed::new()
        .title("Not Registered")
        .description(
            "You need to link your Discord account to a Minecraft account before using this command.\n\nRun `/register_minecraft` to get started.",
        )
        .colour(Colour::RED)
}
