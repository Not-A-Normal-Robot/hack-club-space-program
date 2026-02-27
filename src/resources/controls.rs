use core::fmt::Display;

use bevy::{platform::collections::HashMap, prelude::*};
use derive_more::with_trait::IsVariant;

use crate::{fl, resources::scene::GameScene};

/// An enum determining how to interpret inputs, akin to Vim's different modes.
///
/// Only affects in-game.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, SubStates, IsVariant)]
#[source(GameScene = GameScene::InGame)]
pub(crate) enum GameControlMode {
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

impl Display for GameControlMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Main => f.write_str(&fl!("gameControlMode__mainMode")),
            Self::Menu => f.write_str(&fl!("gameControlMode__menuMode")),
            Self::VesselControl => f.write_str(&fl!("gameControlMode__vesselControlMode")),
            Self::CameraControl => f.write_str(&fl!("gameControlMode__cameraControlMode")),
        }
    }
}

pub(crate) struct FocusableEntry {
    pub(crate) entity: Entity,
    pub(crate) is_celestial_body: bool,
}

#[derive(Default, Resource)]
pub(crate) struct FocusableData {
    pub(crate) index_map: HashMap<Entity, usize>,
    pub(crate) list: Vec<FocusableEntry>,
}
