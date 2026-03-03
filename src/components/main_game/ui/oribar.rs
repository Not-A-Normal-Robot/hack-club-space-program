//! Oribar: Orientation Bar

use crate::{assets::icons::URI_ICON_PROGRADE, resources::scene::GameScene};
use bevy::prelude::*;
use bevy_vello::prelude::VelloSvg;
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
    /// Returns the twin icon URI of the given overlay.
    ///
    /// It goes (positive, negative), e.g. (prograde, retrograde).
    pub(crate) const fn get_icon_uri_set(self) -> (&'static str, &'static str) {
        match self {
            Self::Prograde => {
                (
                    URI_ICON_PROGRADE,
                    "", // TODO: retrograde icon
                )
            }
        }
    }

    pub(crate) fn get_icon_set(
        self,
        asset_server: &AssetServer,
    ) -> (Handle<VelloSvg>, Handle<VelloSvg>) {
        let (pos, neg) = self.get_icon_uri_set();

        let [pos, neg] = [pos, neg].map(|uri| asset_server.load(uri));

        (pos, neg)
    }
}
