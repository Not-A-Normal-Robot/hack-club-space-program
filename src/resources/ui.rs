use bevy::prelude::*;
use strum::EnumIter;

use crate::resources::scene::GameScene;

/// The reference frame that this altimeter is using for its measurements.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumIter, Hash, SubStates)]
#[source(GameScene = GameScene::InGame)]
pub(crate) enum AltimeterMode {
    AboveGroundLevel,
    #[default]
    AboveSeaLevel,
    FromCentre,
}
