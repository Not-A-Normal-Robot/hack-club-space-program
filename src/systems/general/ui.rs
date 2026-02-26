use bevy::{ecs::query::QueryData, prelude::*};

use crate::components::ui::{ActiveTextColor, HoverTextColor, InactiveTextColor};

#[derive(QueryData)]
#[query_data(mutable)]
pub struct InteractableText {
    text_color: &'static mut TextColor,
    inactive_color: &'static InactiveTextColor,
    hover_color: Option<&'static HoverTextColor>,
    active_color: Option<&'static ActiveTextColor>,
    interaction: &'static Interaction,
}

pub fn update_interacted_text_colors(query: Query<InteractableText, Changed<Interaction>>) {
    for mut text in query {
        let new_color = match *text.interaction {
            Interaction::Pressed => text
                .active_color
                .map(|c| c.0)
                .or(text.hover_color.map(|c| c.0))
                .unwrap_or(text.inactive_color.0),
            Interaction::Hovered => text.hover_color.map_or(text.inactive_color.0, |c| c.0),
            Interaction::None => text.inactive_color.0,
        };

        if new_color != text.text_color.0 {
            text.text_color.0 = new_color;
        }
    }
}
