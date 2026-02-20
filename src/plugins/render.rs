use bevy::prelude::*;

use crate::systems::terrain::update_terrain_gfx;

pub struct GameRenderPlugin;

impl Plugin for GameRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_terrain_gfx);
    }
}
