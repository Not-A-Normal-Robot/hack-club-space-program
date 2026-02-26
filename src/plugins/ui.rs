use crate::systems::general::ui::{update_interacted_text_colors, update_tab_focus};
use bevy::prelude::*;

pub(crate) struct MyUiPlugin;

impl Plugin for MyUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_interacted_text_colors, update_tab_focus));
    }
}
