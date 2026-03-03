use bevy::prelude::*;

use crate::{
    resources::scene::GameScene,
    systems::main_game::ui::oribar::{
        self, apply_oribar_state, calculate_oribar_state, init_oribar,
    },
};

#[derive(Component)]
pub(crate) struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScene::InGame), init_oribar);
        app.add_systems(
            Update,
            (
                calculate_oribar_state.pipe(apply_oribar_state),
                oribar::handle_resize,
            )
                .run_if(in_state(GameScene::InGame)),
        );
    }
}
