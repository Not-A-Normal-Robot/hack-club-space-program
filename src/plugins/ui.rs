use crate::systems::general::{
    dynamic_text_color::update_interacted_text_colors, tab_focus::update_tab_focus,
    ui_activation::activation_observer_adder,
};
use bevy::prelude::*;

pub(crate) struct MyUiPlugin;

impl Plugin for MyUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_interacted_text_colors,
                update_tab_focus,
                activation_observer_adder,
            ),
        );
    }
}
