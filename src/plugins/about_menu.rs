use crate::{
    resources::scene::GameScene,
    systems::about_menu::{AboutTab, handle_resize, handle_tab_switch, init_about_menu},
};
use bevy::prelude::*;

pub(crate) struct AboutMenuPlugin;

impl Plugin for AboutMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<AboutTab>();

        app.add_systems(OnEnter(GameScene::AboutMenu), init_about_menu);
        app.add_systems(
            Update,
            (
                handle_resize.run_if(in_state(GameScene::AboutMenu)),
                handle_tab_switch.run_if(state_changed::<AboutTab>),
            ),
        );
    }
}
