pub mod camera;

use crate::resources::GameControlMode;
use bevy::prelude::*;

pub fn insert_control_mode(mut commands: Commands) {
    commands.insert_resource(GameControlMode::Main);
}

pub fn control_switching(_mode: ResMut<GameControlMode>) {
    // TODO
}
