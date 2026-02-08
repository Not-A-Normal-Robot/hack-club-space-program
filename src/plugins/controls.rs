use bevy::prelude::*;

use crate::{
    resources::GameControlMode,
    systems::{controls::control_switching, ui::controls::update_controls_text},
};

pub struct GameControlPlugin;

impl Plugin for GameControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameControlMode>();
        app.add_systems(Update, (control_switching, update_controls_text));
    }
}
