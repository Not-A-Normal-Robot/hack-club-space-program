use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

use crate::{assets::fonts::URI_FONT_WDXL_LUBRIFONT_SC, resources::scene::GameScene};

#[derive(Clone, Copy, Component)]
#[require(DespawnOnExit::<GameScene>(GameScene::MainMenu))]
pub struct MainMenuRootNode;

#[derive(Clone, Copy, Component)]
struct PlayButton;

#[derive(Clone, Copy, Component)]
struct QuitButton;

fn main_menu_button(
    marker: impl Component,
    text: impl Into<String>,
    font: Handle<Font>,
) -> impl Bundle {
    (
        marker,
        Node {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            min_width: Val::Px(48.0),
            min_height: Val::Px(48.0),
            ..Node::DEFAULT
        },
        Button,
        BackgroundColor(Color::Srgba(Srgba::RED)),
        children![(Text::new(text), TextFont::from(font))],
    )
}

fn root_margin(window_size: Vec2) -> UiRect {
    if window_size.x < 960.0 {
        UiRect::horizontal(Val::Auto)
    } else {
        UiRect::left(Val::Vw(10.0))
    }
}

pub fn init_main_menu(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    assets: Res<AssetServer>,
) {
    let font = assets.load::<Font>(URI_FONT_WDXL_LUBRIFONT_SC);

    let play_button = commands
        .spawn(main_menu_button(PlayButton, "Play", font.clone()))
        .observe(
            |_: On<Pointer<Click>>, mut scene: ResMut<NextState<GameScene>>| {
                scene.set(GameScene::InGame);
            },
        )
        .id();

    let quit_button = commands
        .spawn(main_menu_button(QuitButton, "Quit", font))
        .observe(|_: On<Pointer<Click>>| std::process::exit(0))
        .id();

    let root = (
        MainMenuRootNode,
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            max_width: Val::Px(480.0),
            margin: root_margin(window.size()),
            width: Val::Vw(100.0),
            height: Val::Vh(100.0),
            row_gap: Val::Px(16.0),
            ..Default::default()
        },
        BackgroundColor(Color::Srgba(Srgba::BLUE)),
    );

    commands
        .spawn(root)
        .add_children([play_button, quit_button].as_slice());

    commands.spawn((
        DespawnOnExit(GameScene::MainMenu),
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            is_active: true,
            ..Default::default()
        },
        IsDefaultUiCamera,
    ));
}

pub fn handle_resize(
    mut root: Single<&mut Node, With<MainMenuRootNode>>,
    mut resize_reader: MessageReader<WindowResized>,
) {
    let Some(resize) = resize_reader.read().last() else {
        return;
    };

    let cur_margin = root.margin;
    let new_margin = root_margin(Vec2::new(resize.width, resize.height));

    if new_margin != cur_margin {
        root.margin = new_margin;
    }
}
