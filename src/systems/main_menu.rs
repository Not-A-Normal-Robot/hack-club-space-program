use crate::{
    assets::fonts::URI_FONT_DOTO_ROUNDED_BOLD,
    builders::{button::TextButtonBuilder, camera::UiCameraBuilder},
    consts::colors::{
        scheme::SURFACE,
        shades::{PRIMARY_50, PRIMARY_60, PRIMARY_80, TERTIARY_50, TERTIARY_60, TERTIARY_80},
    },
    fl,
    resources::scene::GameScene,
    systems::general::ui_activation::ActivationEvent,
};
use bevy::{
    input_focus::tab_navigation::{TabGroup, TabIndex},
    prelude::*,
    text::LineHeight,
    window::{PrimaryWindow, WindowResized},
};

#[derive(Clone, Copy, Component)]
#[require(DespawnOnExit::<GameScene>(GameScene::MainMenu))]
pub(crate) struct MainMenuRootNode;

fn logo(font: &Handle<Font>) -> impl Bundle {
    (
        Text::new("hack club\nspace program"),
        TextFont::from(font.clone()).with_font_size(48.0),
        LineHeight::RelativeToFont(0.8),
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

fn play_button(extra: impl Bundle, font: &TextFont, commands: &mut Commands) -> Entity {
    let play_button = TextButtonBuilder {
        extra,
        text_extra: (),
        text: fl!("mainMenu__playButton__text"),
        font,
        color: TERTIARY_60,
        hover_color: TERTIARY_80,
        active_color: TERTIARY_50,
    }
    .build();

    commands
        .spawn(play_button)
        .observe(
            |_: On<ActivationEvent>, mut scene: ResMut<NextState<GameScene>>| {
                scene.set(GameScene::InGame);
            },
        )
        .id()
}

fn about_button(extra: impl Bundle, font: &TextFont, commands: &mut Commands) -> Entity {
    let about_button = TextButtonBuilder {
        extra,
        text_extra: (),
        text: fl!("mainMenu__aboutButton__text"),
        font,
        color: PRIMARY_60,
        hover_color: PRIMARY_80,
        active_color: PRIMARY_50,
    }
    .build();

    commands
        .spawn(about_button)
        .observe(
            |_: On<ActivationEvent>, mut scene: ResMut<NextState<GameScene>>| {
                scene.set(GameScene::AboutMenu);
            },
        )
        .id()
}

#[cfg(not(target_family = "wasm"))]
fn quit_button(extra: impl Bundle, font: &TextFont, commands: &mut Commands) -> Entity {
    let quit_button = TextButtonBuilder {
        extra,
        text_extra: (),
        text: fl!("mainMenu__quitButton__text"),
        font,
        color: PRIMARY_60,
        hover_color: PRIMARY_80,
        active_color: PRIMARY_50,
    }
    .build();

    commands
        .spawn(quit_button)
        .observe(|_: On<ActivationEvent>| {
            std::process::exit(0);
        })
        .id()
}

fn root_node(window: &Window, children: &[Entity], commands: &mut Commands) -> Entity {
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
        TabGroup::new(0),
    );

    commands.spawn(root).add_children(children).id()
}

pub(crate) fn init_main_menu(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    assets: Res<AssetServer>,
) {
    let doto_font = assets.load::<Font>(URI_FONT_DOTO_ROUNDED_BOLD);

    let logo = commands.spawn(logo(&doto_font)).id();

    let doto_font = TextFont::from(doto_font).with_font_size(32.0);

    let button_common = (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            min_width: Val::Px(48.0),
            min_height: Val::Px(48.0),
            ..Node::DEFAULT
        },
        TabIndex(0),
    );

    let play_button = play_button(button_common.clone(), &doto_font, &mut commands);
    let about_button = about_button(
        {
            #[cfg(target_family = "wasm")]
            let x = button_common;
            #[cfg(not(target_family = "wasm"))]
            let x = button_common.clone();
            x
        },
        &doto_font,
        &mut commands,
    );

    #[cfg(not(target_family = "wasm"))]
    let quit_button = quit_button(button_common.clone(), &doto_font, &mut commands);

    let root_children = [
        logo,
        play_button,
        about_button,
        #[cfg(not(target_family = "wasm"))]
        quit_button,
    ];

    root_node(*window, root_children.as_slice(), &mut commands);

    commands.spawn(UiCameraBuilder::with_extra((
        DespawnOnExit(GameScene::MainMenu),
        Camera {
            clear_color: ClearColorConfig::Custom(SURFACE),
            is_active: true,
            ..Default::default()
        },
    )));
}

pub(crate) fn handle_resize(
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
