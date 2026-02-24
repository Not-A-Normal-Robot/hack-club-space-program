use bevy::prelude::*;

use crate::{
    resources::{FocusableData, GameControlMode},
    systems::{
        controls::{
            camera::{control_camera, update_focusable_data},
            control_switching,
        },
        ui::controls::update_controls_text,
    },
};

pub struct GameControlPlugin;

impl Plugin for GameControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameControlMode>();
        app.init_resource::<FocusableData>();
        app.add_systems(
            Update,
            (
                (control_switching, update_controls_text),
                (control_camera, update_focusable_data)
                    .run_if(|state: Res<State<GameControlMode>>| state.get().is_camera_control()),
            ),
        );
    }
}
