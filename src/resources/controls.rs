use bevy::{platform::collections::HashMap, prelude::*};
use derive_more::with_trait::IsVariant;

use crate::resources::scene::GameScene;

/// An enum determining how to interpret inputs, akin to Vim's different modes.
///
/// Only affects in-game.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, SubStates, IsVariant)]
#[source(GameScene = GameScene::InGame)]
pub enum GameControlMode {
    /// The "hub" control mode that allows switching to other control modes.
    ///
    /// Every mode can go to the main mode by pressing `Esc`.
    #[default]
    Main,
    /// The mode that allows selecting menus.
    Menu,
    /// The mode that allows controlling the vessel.
    VesselControl,
    /// The mode that allows controlling the camera.
    CameraControl,
}

impl GameControlMode {
    #[must_use]
    pub const fn to_str(self) -> &'static str {
        match self {
            Self::Main => "main mode",
            Self::Menu => "menu mode",
            Self::VesselControl => "vessel control mode",
            Self::CameraControl => "camera control mode",
        }
    }
}

pub struct FocusableEntry {
    pub entity: Entity,
    pub is_celestial_body: bool,
}

#[derive(Default, Resource)]
pub struct FocusableData {
    pub index_map: HashMap<Entity, usize>,
    pub list: Vec<FocusableEntry>,
}
