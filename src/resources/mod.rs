use bevy::prelude::*;

use crate::components::frames::{RootSpaceLinearVelocity, RootSpacePosition};

#[derive(Resource)]
pub struct ActiveVessel {
    pub entity: Entity,
    pub prev_tick_position: RootSpacePosition,
    pub prev_tick_velocity: RootSpaceLinearVelocity,
    pub prev_tick_parent: Entity,
}

/// An enum determining how to interpret inputs, akin to Vim's different modes.
///
/// Only affects in-game.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, States)]
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
    pub const fn to_str(self) -> &'static str {
        match self {
            Self::Main => "main mode",
            Self::Menu => "menu mode",
            Self::VesselControl => "vessel control mode",
            Self::CameraControl => "camera control mode",
        }
    }
}
