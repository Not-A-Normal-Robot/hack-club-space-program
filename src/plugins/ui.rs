use crate::systems::general::ui::update_interacted_text_colors;
use bevy::prelude::*;

pub struct MyUiPlugin;

impl Plugin for MyUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_interacted_text_colors);
    }
}
