pub mod camera;

use crate::{
    consts::keybinds::{
        KB_MODE_SWITCH_TO_CAM_MODE, KB_MODE_SWITCH_TO_MAIN_MODE, KB_MODE_SWITCH_TO_MENU_MODE,
        KB_MODE_SWITCH_TO_VESSEL_MODE,
    },
    resources::GameControlMode,
};
use bevy::prelude::*;

macro_rules! mode_switches {
    ($keyboard:expr, $next_mode:expr, []) => {};
    ($keyboard:expr, $next_mode:expr, [$keycode:expr => $mode:expr $(, $( $rest:tt )* )? ]) => {
        if $keyboard.any_just_pressed($keycode) {
            $next_mode.set($mode);
        }

        $(mode_switches!($keyboard, $next_mode, [$( $rest )*]);)?
    };
}

pub fn control_switching(
    mode: Res<State<GameControlMode>>,
    mut next_mode: ResMut<NextState<GameControlMode>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let is_main = *mode.get() == GameControlMode::Main;

    if keyboard.any_just_pressed(KB_MODE_SWITCH_TO_MAIN_MODE) && !is_main {
        next_mode.set(GameControlMode::Main);
    }

    if !is_main {
        return;
    }

    mode_switches! {
        keyboard,
        next_mode,
        [
            KB_MODE_SWITCH_TO_MENU_MODE => GameControlMode::Menu,
            KB_MODE_SWITCH_TO_VESSEL_MODE => GameControlMode::VesselControl,
            KB_MODE_SWITCH_TO_CAM_MODE => GameControlMode::CameraControl,
        ]
    }
}
