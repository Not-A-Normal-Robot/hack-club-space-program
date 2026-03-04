use bevy::prelude::*;

use crate::{
    assets::fonts::URI_FONT_DOTO_BOLD,
    components::main_game::ui::altimeter::{AltimeterAltitudeText, AltimeterPrefix, AltimeterSign},
    consts::{
        colors::{
            ALTIMETER_ACTIVE, ALTIMETER_BACKGROUND, ALTIMETER_INACTIVE, ALTIMETER_INNER_BORDER,
            ALTIMETER_OUTER_BORDER, ALTIMETER_PREFIX,
        },
        ui::altimeter::ALTIMETER_BIG_TEXT_SIZE,
    },
    resources::scene::GameScene,
};

fn root(children: &[Entity], commands: &mut Commands) -> Entity {
    let inner = commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.5)).with_top(Val::ZERO),
                ..Default::default()
            },
            BackgroundColor(ALTIMETER_BACKGROUND),
            BorderColor::all(ALTIMETER_OUTER_BORDER),
        ))
        .add_children(children)
        .id();
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                top: Val::ZERO,
                left: Val::ZERO,
                width: Val::Vw(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            DespawnOnExit(GameScene::InGame),
        ))
        .add_child(inner)
        .id()
}

#[must_use]
fn altitude(doto_bold: &Handle<Font>, commands: &mut Commands) -> Entity {
    let doto_bold = TextFont::from(doto_bold.clone()).with_font_size(ALTIMETER_BIG_TEXT_SIZE);

    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..Default::default()
            },
            BorderColor::all(ALTIMETER_INNER_BORDER),
            children![
                (
                    Text("-".into()),
                    doto_bold.clone(),
                    TextColor(ALTIMETER_INACTIVE),
                    AltimeterSign,
                ),
                (
                    Node {
                        margin: UiRect::right(Val::Px(16.0)),
                        ..Default::default()
                    },
                    Text("000000".into()),
                    doto_bold.clone(),
                    TextColor(ALTIMETER_ACTIVE),
                    AltimeterAltitudeText,
                ),
                (
                    Text("m".into()),
                    doto_bold,
                    TextColor(ALTIMETER_PREFIX),
                    AltimeterPrefix,
                ),
            ],
        ))
        .id()
}

pub(crate) fn init_altimeter(mut commands: Commands, server: Res<AssetServer>) {
    let doto_bold = server.load::<Font>(URI_FONT_DOTO_BOLD);

    let altitude = altitude(&doto_bold, &mut commands);
    root(&[altitude], &mut commands);
}
