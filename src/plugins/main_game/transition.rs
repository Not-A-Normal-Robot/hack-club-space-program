use crate::{
    resources::scene::GameScene,
    systems::main_game::transition::{exit_game, load_game},
};
use bevy::prelude::*;

/// Handles transitions to and from the [`GameScene::InGame`] scene.
pub(crate) struct GameTransitionPlugin;
impl Plugin for GameTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScene::InGame), load_game);
        app.add_systems(OnExit(GameScene::InGame), exit_game);
    }
}
