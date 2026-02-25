use crate::{
    assets::fonts::initialize_fonts,
    consts::WEB_CANVAS_SELECTOR,
    plugins::{
        main_game::{
            controls::GameControlPlugin, debug::GameDebugPlugin, gfx::GameGfxPlugin,
            logic::GameLogicPlugin, transition::GameTransitionPlugin,
        },
        main_menu::MainMenuPlugin,
    },
    resources::scene::GameScene,
};
#[cfg(feature = "trace")]
use bevy::log::Level;
use bevy::{log::LogPlugin, prelude::*};

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
        initialize_fonts(app);
        app.init_state::<GameScene>();
        app.add_plugins((
            MainMenuPlugin,
            GameLogicPlugin,
            GameTransitionPlugin,
            GameDebugPlugin,
            GameControlPlugin,
            GameGfxPlugin,
        ));
    }
}
