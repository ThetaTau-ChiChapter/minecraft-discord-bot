use poise::serenity_prelude::{FullEvent, Interaction};

use crate::{bot::commands, state::State};


/// Main event handler
#[dbutton_macro::event_handler(
    crate::bot::commands::send_new_code,
    crate::bot::commands::enter_code
)]
pub async fn event_handler(
    ctx: &poise::serenity_prelude::Context,
    event: &FullEvent,
    framework: poise::FrameworkContext<'_, State, crate::Error>,
    state: &State,
) -> Result<(), crate::Error> {
    // Modal submissions arrive as their own interaction variant, so `#[dbutton]` (which only
    // routes `Interaction::Component`) can't handle them - dispatch by custom id here instead.
    if let FullEvent::InteractionCreate { interaction } = event
        && let Interaction::Modal(modal) = interaction
        && modal.data.custom_id == commands::ENTER_CODE_MODAL_ID
    {
        return commands::handle_enter_code_modal_submit(ctx, modal, state).await;
    }

    Ok(())
}