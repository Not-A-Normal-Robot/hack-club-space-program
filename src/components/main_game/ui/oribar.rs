//! Oribar: Orientation Bar

use crate::{assets::icons::ICON_PROGRADE, resources::scene::GameScene};
use bevy::prelude::*;
use bevy_prototype_lyon::entity::Shape;
use strum::{EnumCount, EnumIter};

#[derive(Component)]
#[require(DespawnOnExit::<GameScene>(GameScene::InGame), Node)]
pub(crate) struct Oribar;

#[derive(Component)]
#[require(DespawnOnExit::<GameScene>(GameScene::InGame), Node)]
pub(crate) struct OribarIndicator;

#[derive(Clone, Copy, Component, Debug, PartialEq, Eq, Hash, EnumCount, EnumIter)]
pub(crate) enum OribarOverlay {
    /// The prograde and retrograde overlay (combined into one element).
    Prograde,
}

impl OribarOverlay {
    /// Returns the twin icon of the given overlay.
    ///
    /// It goes (positive, negative), e.g. (prograde, retrograde).
    pub(crate) fn get_icon_set(self) -> (Shape, Shape) {
        match self {
            Self::Prograde => (
                ICON_PROGRADE.clone(),
                ICON_PROGRADE.clone(), // TODO: retrograde icon
            ),
        }
    }
}
