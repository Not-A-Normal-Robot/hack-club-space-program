use crate::resources::scene::GameScene;
use bevy::prelude::*;

#[derive(Component)]
#[require(DespawnOnExit::<GameScene>(GameScene::InGame))]
pub(crate) struct ControlsText;
