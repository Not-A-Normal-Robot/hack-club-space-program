use bevy::prelude::*;

use crate::components::ui::{ActiveTextColor, HoverTextColor, InactiveTextColor};

pub struct ButtonBuilder<'f, E, T>
where
    E: Bundle,
    T: Into<String>,
{
    pub extra: E,
    pub text: T,
    pub font: &'f TextFont,
    pub color: Color,
    pub hover_color: Color,
    pub active_color: Color,
}

impl<E, T> ButtonBuilder<'_, E, T>
where
    E: Bundle,
    T: Into<String>,
{
    pub fn build(self) -> impl Bundle {
        (
            self.extra,
            Button,
            Text::new(self.text),
            self.font.clone(),
            TextColor(self.color),
            InactiveTextColor(self.color),
            HoverTextColor(self.hover_color),
            ActiveTextColor(self.active_color),
        )
    }
}
