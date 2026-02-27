use bevy::prelude::*;

use crate::{
    resources::{controls::GameControlMode, scene::GameScene},
    systems::main_game::{
        controls::{
            camera::{control_camera, update_focusable_data},
            cleanup_controls, control_switching, init_controls,
        },
        ui::controls::update_controls_text,
    },
};

pub(crate) struct GameControlPlugin;

impl Plugin for GameControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<GameControlMode>();
        app.add_systems(OnEnter(GameScene::InGame), init_controls);
        app.add_systems(OnExit(GameScene::InGame), cleanup_controls);
        app.add_systems(
            Update,
            (
                control_switching,
                update_controls_text.run_if(state_changed::<GameControlMode>),
                (control_camera, update_focusable_data)
                    .run_if(in_state(GameControlMode::CameraControl)),
            )
                .run_if(in_state(GameScene::InGame)),
        );
    }
}
