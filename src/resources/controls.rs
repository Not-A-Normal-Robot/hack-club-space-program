use core::{fmt::Display, ops::Range};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct FocusableEntry {
    pub(crate) entity: Entity,
    pub(crate) is_celestial_body: bool,
}

#[derive(Clone, Default, Debug, Resource, PartialEq, Eq)]
pub(crate) struct FocusableData {
    index_map: HashMap<Entity, usize>,
    focusable_list: Vec<FocusableEntry>,
}

impl FocusableData {
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

    /// Insert an entry at the end of the list.
    ///
    /// If the entry is already in the list, it will instead
    /// be moved to the new index.
    pub(crate) fn push(&mut self, entry: FocusableEntry) {
        self.insert(self.len(), entry);
    }

    /// Updates the index map for the selected index range.
    ///
    /// # Panics
    /// Panics if `index_range` goes out of range.
    fn update_index_maps(&mut self, index_range: Range<usize>) {
        for index in index_range {
            let entity = self.focusable_list()[index].entity;

            if let Some(value) = self.index_map.get_mut(&entity) {
                *value = index;
            } else {
                self.index_map.insert(entity, index);
            }
        }
    }

    /// Moves an existing entry to a new index, shifting the rest of
    /// the entries along the way.
    ///
    /// # Panics
    /// Panics when:
    /// - `new_index >= len`, or
    /// - `old_index >= len`, or
    /// - `len = 0`
    fn swap_shift(&mut self, old_index: usize, new_index: usize) {
        let old = self.focusable_list[old_index];

        if old_index < new_index {
            for i in old_index..new_index {
                let next = self.focusable_list[i + 1];
                self.focusable_list[i] = next;
            }
        } else {
            for i in (new_index + 1..=old_index).rev() {
                let prev = self.focusable_list[i - 1];
                self.focusable_list[i] = prev;
            }
        }

        self.focusable_list[new_index] = old;

        let mut ends = [old_index, new_index];
        ends.sort_unstable();

        let modified_range = ends[0]..ends[1] + 1;

        self.update_index_maps(modified_range);
    }

    /// Insert an entry at a given index, shifting the entries after that index
    /// to the right.
    ///
    /// If the entry is already in the list, it will instead be moved to the new index.
    ///
    /// # Panics
    /// Panics if:
    /// - The entry is not in the data structure and `index > len`
    /// - The entry is in the data structure and `index >= len`
    pub(crate) fn insert(&mut self, index: usize, entry: FocusableEntry) {
        if let Some(&old_index) = self.index_map.get(&entry.entity) {
            self.swap_shift(old_index, index);
            return;
        }

        self.focusable_list.insert(index, entry);
        let modified_range = index..self.focusable_list().len();

        self.update_index_maps(modified_range);
    }

    /// Removes an entry at a given index, shifting the entries after that index
    /// to the left.
    ///
    /// # Panics
    /// Panics if `index >= len`.
    pub(crate) fn remove(&mut self, index: usize) -> FocusableEntry {
        let entry = self.focusable_list.remove(index);
        self.index_map.remove(&entry.entity);

        self.update_index_maps(index..self.focusable_list().len());

        entry
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::entity::{EntityGeneration, EntityIndex};

    use super::*;

    impl FocusableData {
        /// Checks if this data structure's invariant is maintained.
        ///
        /// Namely, this checks if the bijective property is upheld.
        fn integrity_check(&self) {
            assert_eq!(
                self.index_map().len(),
                self.focusable_list().len(),
                "List and map have unequal lengths"
            );

            for (&map_entity, &map_index) in self.index_map() {
                let entry = self
                    .focusable_list()
                    .get(map_index)
                    .copied()
                    .expect("list should contain what the map says it contains");

                assert_eq!(
                    entry.entity, map_entity,
                    "List's entity and map's entity doesn't match"
                );
            }
        }
    }

    /// Converts an integer to a focusableentry for testing and generation purposes.
    fn int_to_entry(int: u64) -> FocusableEntry {
        let top_dword = (int >> 32) as u32;
        let bottom_dword = (int & 0xFFFF) as u32;
        FocusableEntry {
            is_celestial_body: int.is_multiple_of(2),
            entity: Entity::from_index_and_generation(
                EntityIndex::new(top_dword.try_into().unwrap()),
                EntityGeneration::from_bits(bottom_dword),
            ),
        }
    }

    #[test]
    fn foc_data_trivial_insert() {
        let mut data_insert = FocusableData::default();

        assert!(data_insert.is_empty());
        assert_eq!(data_insert.len(), 0);

        data_insert.insert(0, int_to_entry(0));

        assert!(!data_insert.is_empty());
        assert_eq!(data_insert.len(), 1);
        data_insert.integrity_check();

        let mut data_push = FocusableData::default();

        data_push.push(int_to_entry(0));

        assert!(!data_push.is_empty());
        assert_eq!(data_push.len(), 1);
        data_push.integrity_check();

        assert_eq!(data_insert, data_push);
    }

    #[test]
    fn foc_data_middle_insert() {
        let mut data = FocusableData::default();

        for i in 0..4 {
            data.insert(i, int_to_entry(i as u64));
            data.integrity_check();
        }

        assert_eq!(
            data.focusable_list,
            (0..4).map(int_to_entry).collect::<Vec<_>>()
        );

        data.insert(3, int_to_entry(5));
        data.integrity_check();

        assert_eq!(
            data.focusable_list,
            [0, 1, 2, 5, 3].map(int_to_entry).as_slice()
        );

        data.insert(1, int_to_entry(6));
        data.integrity_check();

        assert_eq!(
            data.focusable_list,
            [0, 6, 1, 2, 5, 3].map(int_to_entry).as_slice()
        );

        data.insert(0, int_to_entry(7));
        data.integrity_check();

        assert_eq!(
            data.focusable_list,
            [7, 0, 6, 1, 2, 5, 3].map(int_to_entry).as_slice()
        );
    }

    #[test]
    fn foc_data_swap_insert() {
        let mut data = FocusableData::default();

        for i in 0..4 {
            data.insert(i, int_to_entry(i as u64));
            data.integrity_check();
        }

        data.insert(3, int_to_entry(0));

        assert_eq!(
            data.focusable_list,
            [1, 2, 3, 0].map(int_to_entry).as_slice()
        );
        data.integrity_check();

        data.insert(0, int_to_entry(3));

        assert_eq!(
            data.focusable_list,
            [3, 1, 2, 0].map(int_to_entry).as_slice()
        );
        data.integrity_check();
    }

    #[test]
    #[should_panic = "insertion index (is 5) should be <= len (is 4)"]
    fn foc_data_insert_nonexistent_panic() {
        let mut data = FocusableData::default();

        for i in 0..4 {
            data.insert(i, int_to_entry(i as u64));
            data.integrity_check();
        }

        data.insert(5, int_to_entry(42));
    }

    #[test]
    #[should_panic = "index out of bounds"]
    fn foc_data_insert_existent_panic() {
        let mut data = FocusableData::default();

        for i in 0..4 {
            data.insert(i, int_to_entry(i as u64));
            data.integrity_check();
        }

        data.insert(4, int_to_entry(2));
    }

    #[test]
    fn foc_data_insert_noop() {
        let mut data = FocusableData::default();

        for i in 0..32 {
            data.insert(i, int_to_entry(i as u64));
            data.integrity_check();
        }

        let old_data = data.clone();

        for _ in 0..3 {
            for i in 0..32 {
                data.insert(i, int_to_entry(i as u64));
                data.integrity_check();
            }
        }

        assert_eq!(old_data, data);
    }

    #[test]
    fn foc_data_removal() {
        let mut data = FocusableData::default();

        for i in 0..4 {
            data.insert(i, int_to_entry(i as u64));
            data.integrity_check();
        }

        let old_data = data.clone();

        let removed = data.remove(data.len() - 1);
        data.integrity_check();

        assert_eq!(
            data.focusable_list(),
            [0, 1, 2].map(int_to_entry).as_slice()
        );
        assert_eq!(removed, int_to_entry(3));

        let removed = data.remove(0);
        data.integrity_check();

        assert_eq!(data.focusable_list(), [1, 2].map(int_to_entry).as_slice());
        assert_eq!(removed, int_to_entry(0));

        data.insert(0, int_to_entry(0));
        data.integrity_check();

        data.insert(data.len(), int_to_entry(data.len() as u64));
        data.integrity_check();

        assert_eq!(old_data, data);
        drop(old_data);

        let removed = data.remove(1);
        data.integrity_check();

        assert_eq!(
            data.focusable_list(),
            [0, 2, 3].map(int_to_entry).as_slice()
        );
        assert_eq!(removed, int_to_entry(1));

        for _ in 0..3 {
            data.remove(0);
            data.integrity_check();
        }

        assert!(data.is_empty());
    }
}
