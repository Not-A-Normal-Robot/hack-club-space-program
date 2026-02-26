use bevy::prelude::*;

use crate::{resources::scene::GameScene, systems::main_game::terrain::gfx::update_terrain_gfx};

pub struct GameGfxPlugin;

impl Plugin for GameGfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_terrain_gfx.run_if(in_state(GameScene::InGame)),
        );
    }
}
