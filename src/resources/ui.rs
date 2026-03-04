use bevy::prelude::*;
use strum::EnumIter;

use crate::resources::scene::GameScene;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumIter, Hash, SubStates)]
#[source(GameScene = GameScene::InGame)]
pub(crate) enum AltimeterState {
    AboveGroundLevel,
    #[default]
    AboveSeaLevel,
    FromCentre,
}
