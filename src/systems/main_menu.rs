use crate::{
    assets::fonts::{URI_FONT_DOTO_ROUNDED_BOLD, URI_FONT_WDXL_LUBRIFONT_SC},
    builders::button::ButtonBuilder,
    consts::colors::shades::{
        PRIMARY_50, PRIMARY_60, PRIMARY_70, TERTIARY_30, TERTIARY_50, TERTIARY_60, TERTIARY_70,
    },
    resources::scene::GameScene,
};
use bevy::{
    prelude::*,
    text::LineHeight,
    window::{PrimaryWindow, WindowResized},
};

#[derive(Clone, Copy, Component)]
#[require(DespawnOnExit::<GameScene>(GameScene::MainMenu))]
pub struct MainMenuRootNode;

#[derive(Clone, Copy, Component)]
struct PlayButton;

#[derive(Clone, Copy, Component)]
struct QuitButton;

fn logo(font: &Handle<Font>) -> impl Bundle {
    (
        Text::new("hack club\nspace program"),
        TextFont::from(font.clone())
            .with_font_size(48.0)
            .with_line_height(LineHeight::RelativeToFont(0.8)),
        TextColor(PRIMARY_60),
        Node {
            align_self: AlignSelf::Center,
            margin: UiRect::bottom(Val::Px(48.0)),
            ..Default::default()
        },
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
    let doto_font = assets.load::<Font>(URI_FONT_DOTO_ROUNDED_BOLD);
    // let wdxl_font = assets.load::<Font>(URI_FONT_WDXL_LUBRIFONT_SC);

    let logo = commands.spawn(logo(&doto_font)).id();

    let doto_font = TextFont::from(doto_font).with_font_size(32.0);

    let button_common = (
        Node {
            display: Display::Flex,
            min_width: Val::Px(48.0),
            min_height: Val::Px(48.0),
            ..Node::DEFAULT
        },
        TextLayout {
            justify: Justify::Center,
            linebreak: LineBreak::WordOrCharacter,
        },
    );

    let play_button = ButtonBuilder {
        extra: (
            PlayButton,
            button_common.clone(),
            Outline {
                color: TERTIARY_30,
                width: Val::Px(2.0),
                offset: Val::Px(0.0),
            },
        ),
        text: "Play",
        font: &doto_font,
        color: TERTIARY_60,
        hover_color: TERTIARY_70,
        active_color: TERTIARY_50,
    }
    .build();

    let play_button = commands
        .spawn(play_button)
        .observe(
            |_: On<Pointer<Click>>, mut scene: ResMut<NextState<GameScene>>| {
                scene.set(GameScene::InGame);
            },
        )
        .id();

    let quit_button = ButtonBuilder {
        extra: (
            QuitButton,
            button_common,
            Outline {
                color: TERTIARY_30,
                width: Val::Px(2.0),
                offset: Val::Px(0.0),
            },
        ),
        text: "Quit",
        font: &doto_font,
        color: PRIMARY_60,
        hover_color: PRIMARY_70,
        active_color: PRIMARY_50,
    }
    .build();

    let quit_button = commands
        .spawn(quit_button)
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
            padding: UiRect::horizontal(Val::Px(16.0)),
            width: Val::Vw(100.0),
            height: Val::Vh(100.0),
            row_gap: Val::Px(16.0),
            ..Default::default()
        },
    );

    commands
        .spawn(root)
        .add_children([logo, play_button, quit_button].as_slice());

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
