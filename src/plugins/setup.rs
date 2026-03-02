use crate::{
    assets::fonts::initialize_fonts,
    consts::WEB_CANVAS_SELECTOR,
    plugins::{
        about_menu::AboutMenuPlugin,
        i18n::I18nPlugin,
        main_game::{
            controls::GameControlPlugin, debug::GameDebugPlugin, gfx::GameGfxPlugin,
            logic::GameLogicPlugin, transition::GameTransitionPlugin, ui::GameUiPlugin,
        },
        main_menu::MainMenuPlugin,
        ui::MyUiPlugin,
    },
    resources::scene::GameScene,
};
#[cfg(feature = "trace")]
use bevy::log::Level;
use bevy::{
    input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin},
    log::LogPlugin,
    prelude::*,
};

/// The entry point for the full game as a plugin.
///
/// Automatically initializes all other plugins for the game.
pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    #[cfg(feature = "trace")]
                    level: Level::TRACE,
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        canvas: Some(WEB_CANVAS_SELECTOR.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
        );
        app.add_plugins((InputDispatchPlugin, TabNavigationPlugin));
        initialize_fonts(app);
        app.init_state::<GameScene>();
        app.add_plugins((
            I18nPlugin,
            MainMenuPlugin,
            AboutMenuPlugin,
            MyUiPlugin,
            GameLogicPlugin,
            GameTransitionPlugin,
            GameDebugPlugin,
            GameControlPlugin,
            GameUiPlugin,
            GameGfxPlugin,
        ));
    }
}
