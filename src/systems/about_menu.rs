use bevy::{
    input_focus::tab_navigation::{TabGroup, TabIndex},
    prelude::*,
    window::WindowResized,
};

use crate::{
    assets::fonts::{URI_FONT_DOTO_ROUNDED_BOLD, URI_FONT_WDXL_LUBRIFONT_SC},
    builders::button::ButtonBuilder,
    consts::colors::shades::{PRIMARY_50, PRIMARY_60, PRIMARY_80},
    fl,
    resources::scene::GameScene,
    systems::general::ui_activation::ActivationEvent,
};

#[derive(Component)]
#[require(DespawnOnExit::<GameScene>(GameScene::AboutMenu), TabGroup)]
pub(crate) struct AboutMenuRootNode;

pub(crate) fn init_about_menu(mut commands: Commands, server: Res<AssetServer>) {
    let doto_font = server.load::<Font>(URI_FONT_DOTO_ROUNDED_BOLD);
    let wdxl_font = server.load::<Font>(URI_FONT_WDXL_LUBRIFONT_SC);

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
        TextLayout {
            justify: Justify::Center,
            linebreak: LineBreak::WordOrCharacter,
        },
        TabIndex(0),
    );

    let back_button = ButtonBuilder {
        extra: button_common.clone(),
        text_extra: (),
        text: fl!("aboutMenu__backButton__text"),
        font: &doto_font,
        color: PRIMARY_60,
        hover_color: PRIMARY_80,
        active_color: PRIMARY_50,
    }
    .build();

    let back_button = commands
        .spawn(back_button)
        .observe(
            |_: On<ActivationEvent>, mut scene: ResMut<NextState<GameScene>>| {
                scene.set(GameScene::MainMenu);
            },
        )
        .id();

    commands
        .spawn((
            AboutMenuRootNode,
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Vw(1.0),
                height: Val::Vh(1.0),
                ..Default::default()
            },
        ))
        .add_children([back_button].as_slice());

    commands.spawn((
        DespawnOnExit(GameScene::AboutMenu),
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            is_active: true,
            ..Default::default()
        },
        IsDefaultUiCamera,
    ));
}

pub(crate) fn handle_resize(mut resize_reader: MessageReader<WindowResized>) {
    // TODO
}
