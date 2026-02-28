use crate::{
    resources::scene::GameScene,
    systems::about_menu::{handle_resize, init_about_menu},
};
use bevy::prelude::*;

pub(crate) struct AboutMenuPlugin;

impl Plugin for AboutMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameScene::AboutMenu),
            (init_about_menu, handle_resize),
        );
        app.add_systems(Update, handle_resize.run_if(in_state(GameScene::AboutMenu)));
    }
}
