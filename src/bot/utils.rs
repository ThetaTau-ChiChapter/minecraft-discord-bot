use poise::serenity_prelude::{ActionRowComponent, ButtonKind, ComponentInteraction, CreateActionRow, CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage};

/// Update the interaction's message to disable and label the clicked button "Loading..." while the handler works.
pub async fn defer_response(
    ctx: &poise::serenity_prelude::Context,
    interaction: &ComponentInteraction,
) -> Result<(), crate::Error> {
    let button_id = &interaction.data.custom_id;
    let components: Vec<CreateActionRow> = interaction
        .message
        .components
        .iter()
        .filter_map(|action_row| {
            let components = &action_row.components;

            if let Some(first_component) = components.first() {
                match first_component {
                    ActionRowComponent::Button(_button) => Some(CreateActionRow::Buttons(
                        components
                            .iter()
                            .map(|component| {
                                let ActionRowComponent::Button(button) = component else {
                                    panic!("Expected button component")
                                };
                                let new_button = CreateButton::from(button.clone());
                                if let ButtonKind::NonLink {
                                    custom_id,
                                    style: _,
                                } = &button.data
                                    && *custom_id == *button_id
                                {
                                    return new_button.disabled(true).label("Loading...");
                                }
                                new_button
                            })
                            .collect(),
                    )),
                    ActionRowComponent::InputText(_text) => None,
                    ActionRowComponent::SelectMenu(_menu) => None,
                    &_ => None,
                }
            } else {
                None
            }
        })
        .collect();

    interaction
        .create_response(
            ctx.http.clone(),
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new().components(
                    components, // vec![
                               //     CreateActionRow::Buttons(vec![
                               //         CreateButton::new("Loading").label("Loading...").style(ButtonStyle::Secondary).disabled(true),
                               //     ]),
                               // ]
                ),
            ),
        )
        .await?;

    Ok(())
}
