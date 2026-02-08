use bevy::prelude::*;

use crate::systems::controls::insert_control_mode;

pub struct GameControlPlugin;

impl Plugin for GameControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, insert_control_mode);
        // TODO
    }
}
