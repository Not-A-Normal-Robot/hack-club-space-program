#![cfg_attr(not(feature = "not-headless"), expect(dead_code))]

use bevy::prelude::*;

use crate::components::ui::{ActiveTextColor, HoverTextColor, InactiveTextColor};

pub(crate) struct TextButtonBuilder<'f, E, F, T>
where
    E: Bundle,
    F: Bundle,
    T: Into<String>,
{
    pub(crate) extra: E,
    pub(crate) text_extra: F,
    pub(crate) text: T,
    pub(crate) font: &'f TextFont,
    pub(crate) color: Color,
    pub(crate) hover_color: Color,
    pub(crate) active_color: Color,
}

impl<E, F, T> TextButtonBuilder<'_, E, F, T>
where
    E: Bundle,
    F: Bundle,
    T: Into<String>,
{
    pub(crate) fn build(self) -> impl Bundle {
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
