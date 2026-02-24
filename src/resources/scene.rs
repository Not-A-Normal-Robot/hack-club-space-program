use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, States)]
pub enum GameScene {
    #[default]
    MainMenu,
    InGame,
}
