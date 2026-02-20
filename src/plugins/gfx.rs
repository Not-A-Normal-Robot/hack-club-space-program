use bevy::prelude::*;

use crate::systems::terrain::gfx::update_terrain_gfx;

pub struct GameGfxPlugin;

impl Plugin for GameGfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_terrain_gfx);
    }
}
