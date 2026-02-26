use bevy::{ecs::query::QueryData, prelude::*};

use crate::components::ui::{ActiveTextColor, HoverTextColor, InactiveTextColor};

#[derive(QueryData)]
pub struct DynTextColorData {
    entity: Entity,
    inactive_color: &'static InactiveTextColor,
    hover_color: Option<&'static HoverTextColor>,
    active_color: Option<&'static ActiveTextColor>,
    interaction: &'static Interaction,
    children: &'static Children,
}

pub fn update_interacted_text_colors(
    dyn_query: Query<DynTextColorData, Changed<Interaction>>,
    mut cur_query: Query<(&mut TextColor, Option<&InactiveTextColor>)>,
) {
    for data in dyn_query {
        let new_color = match *data.interaction {
            Interaction::Pressed => data
                .active_color
                .map(|c| c.0)
                .or(data.hover_color.map(|c| c.0))
                .unwrap_or(data.inactive_color.0),
            Interaction::Hovered => data.hover_color.map_or(data.inactive_color.0, |c| c.0),
            Interaction::None => data.inactive_color.0,
        };

        if let Ok((mut color, _)) = cur_query.get_mut(data.entity)
            && new_color != color.0
        {
            color.0 = new_color;
        }

        for &child in data.children {
            let Some(mut color) = cur_query
                .get_mut(child)
                .ok()
                .and_then(|(color, option)| option.is_none().then_some(color))
            else {
                continue;
            };

            if new_color != color.0 {
                color.0 = new_color;
            }
        }
    }
}
