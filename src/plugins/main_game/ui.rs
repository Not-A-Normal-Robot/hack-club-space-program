use bevy::prelude::*;

use crate::{
    resources::{scene::GameScene, ui::AltimeterMode},
    systems::main_game::ui::{
        altimeter::{
            self, apply_altimeter_format, calculate_altitude_format, init_altimeter,
            update_altimeter_ref_disp,
        },
        oribar::{self, apply_oribar_state, calculate_oribar_state, init_oribar},
        speedometer::init_speedometer,
    },
};

#[derive(Component)]
pub(crate) struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<AltimeterMode>();
        app.add_systems(
            OnEnter(GameScene::InGame),
            (init_oribar, init_altimeter, init_speedometer),
        );
        app.add_systems(
            Update,
            (
                calculate_oribar_state.pipe(apply_oribar_state),
                calculate_altitude_format.pipe(apply_altimeter_format),
                update_altimeter_ref_disp.run_if(state_changed::<AltimeterMode>),
                oribar::handle_resize,
                altimeter::handle_resize,
            )
                .run_if(in_state(GameScene::InGame)),
        );
    }
}
