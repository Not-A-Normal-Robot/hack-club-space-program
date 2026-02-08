use bevy::prelude::*;

use crate::{
    resources::GameControlMode,
    systems::{
        controls::{camera::control_camera, control_switching},
        ui::controls::update_controls_text,
    },
};

pub struct GameControlPlugin;

impl Plugin for GameControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameControlMode>();
        app.add_systems(
            Update,
            (
                (control_switching, update_controls_text),
                control_camera
                    .run_if(|state: Res<State<GameControlMode>>| state.get().is_camera_control()),
            ),
        );
    }
}
