//! Oribar: Orientation Bar

use crate::resources::scene::GameScene;
use bevy::prelude::*;

#[derive(Component)]
#[require(DespawnOnExit::<GameScene>(GameScene::InGame), Node)]
pub(crate) struct Oribar;
