use bevy::prelude::*;

use crate::storage::SaveName;

#[derive(Resource)]
pub(crate) struct CurrentSaveName(pub(crate) SaveName);
