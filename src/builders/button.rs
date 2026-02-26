use bevy::prelude::*;

use crate::components::ui::{ActiveTextColor, HoverTextColor, InactiveTextColor};

pub struct ButtonBuilder<'f, E, F, T>
where
    E: Bundle,
    F: Bundle,
    T: Into<String>,
{
    pub extra: E,
    pub text_extra: F,
    pub text: T,
    pub font: &'f TextFont,
    pub color: Color,
    pub hover_color: Color,
    pub active_color: Color,
}

impl<E, F, T> ButtonBuilder<'_, E, F, T>
where
    E: Bundle,
    F: Bundle,
    T: Into<String>,
{
    pub fn build(self) -> impl Bundle {
        (
            self.extra,
            Button,
            InactiveTextColor(self.color),
            HoverTextColor(self.hover_color),
            ActiveTextColor(self.active_color),
            children![(
                self.text_extra,
                Text::new(self.text),
                self.font.clone(),
                TextColor(self.color),
            )],
        )
    }
}
