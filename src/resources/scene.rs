use bevy::prelude::*;
use derive_more::IsVariant;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, IsVariant, States)]
pub enum GameScene {
    #[default]
    MainMenu,
    InGame,
}
