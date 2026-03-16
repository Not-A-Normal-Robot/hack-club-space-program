use bevy::prelude::*;

use crate::systems::storage::{poll_storage_setup, setup_storage};

pub(crate) struct StoragePlugin;

impl Plugin for StoragePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_storage);
        app.add_systems(Update, poll_storage_setup);
    }
}
