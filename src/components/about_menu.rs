use crate::resources::scene::GameScene;
use bevy::{input_focus::tab_navigation::TabGroup, prelude::*};

#[derive(Component)]
#[require(DespawnOnExit::<GameScene>(GameScene::AboutMenu), TabGroup)]
pub(crate) struct RootNode;

#[derive(Component)]
pub(crate) struct BackButton;

#[derive(Component)]
pub(crate) struct HeaderTitle;

#[derive(Component)]
pub(crate) struct MainAsideSeparator;

#[derive(Component)]
pub(crate) struct MainElement;

#[derive(Component)]
pub(crate) struct ArticleElement;

#[derive(Component)]
pub(crate) struct AsideElement;

#[derive(Component)]
pub(crate) struct TabElement(pub(crate) usize);

#[derive(Component)]
pub(crate) struct TabText;

#[derive(Clone, Copy, Debug, Default, SubStates, PartialEq, Eq, Hash)]
#[source(GameScene = GameScene::AboutMenu)]
pub(crate) struct AboutTab(pub(crate) usize);
