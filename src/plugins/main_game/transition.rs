use crate::{
    resources::scene::GameScene,
    systems::main_game::transition::{exit_game, init_game},
};
use bevy::prelude::*;

/// Handles transitions to and from the [`GameScene::InGame`] scene.
pub struct GameTransitionPlugin;
impl Plugin for GameTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScene::InGame), init_game);
        app.add_systems(OnExit(GameScene::InGame), exit_game);
    }
}
