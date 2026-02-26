use bevy::prelude::*;

use crate::{
    resources::scene::GameScene,
    systems::main_menu::{handle_resize, init_main_menu},
};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScene::MainMenu), init_main_menu);
        app.add_systems(
            Update,
            (handle_resize).run_if(in_state(GameScene::MainMenu)),
        );
    }
}
