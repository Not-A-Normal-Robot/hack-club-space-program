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

#[derive(Clone, Copy, Debug)]
pub(crate) struct FocusableEntry {
    pub(crate) entity: Entity,
    pub(crate) is_celestial_body: bool,
}

#[derive(Default, Resource)]
pub(crate) struct FocusableData {
    index_map: HashMap<Entity, usize>,
    focusable_list: Vec<FocusableEntry>,
}

impl FocusableData {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            index_map: HashMap::new(),
            focusable_list: Vec::new(),
        }
    }

    #[must_use]
    pub(crate) const fn index_map(&self) -> &HashMap<Entity, usize> {
        &self.index_map
    }

    #[must_use]
    pub(crate) const fn focusable_list(&self) -> &Vec<FocusableEntry> {
        &self.focusable_list
    }

    #[must_use]
    pub(crate) const fn len(&self) -> usize {
        self.focusable_list().len()
    }

    #[must_use]
    pub(crate) const fn is_empty(&self) -> bool {
        self.focusable_list().is_empty()
    }

    #[must_use]
    pub(crate) fn get_index(&self, entity: Entity) -> Option<usize> {
        self.index_map().get(&entity).copied()
    }

    #[must_use]
    pub(crate) fn get_entry(&self, index: usize) -> Option<FocusableEntry> {
        self.focusable_list().get(index).copied()
    }

    pub(crate) fn insert(&mut self, index: usize, entry: FocusableEntry) {
        todo!("inserting data");
    }

    pub(crate) fn remove(&mut self, index: usize) {
        todo!("removing data");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_focusable_data() {
        todo!("test focusable data");
    }
}
