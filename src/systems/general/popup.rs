use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};

use crate::{
    assets::fonts::{URI_FONT_DOTO_BLACK, URI_FONT_WDXL_LUBRIFONT_SC},
    builders::button::TextButtonBuilder,
    consts::{
        colors::{
            POPUP_BACKGROUND_COLOR, POPUP_BODY_BACKGROUND_COLOR, POPUP_BODY_BORDER_COLOR,
            POPUP_BORDER_COLOR, POPUP_DISMISS_ACTIVE_COLOR, POPUP_DISMISS_COLOR,
            POPUP_DISMISS_HOVER_COLOR, POPUP_TITLE_COLOR,
        },
        ui::popup::{POPUP_BODY_FONT_SIZE, POPUP_BUTTON_FONT_SIZE, POPUP_TITLE_FONT_SIZE},
    },
    fl,
};

pub(crate) struct Popup {
    pub(crate) title: String,
    pub(crate) body: String,
}

#[must_use]
fn title(text: String, font: TextFont, commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..Default::default()
            },
            children![(Text(text), font, TextColor(POPUP_TITLE_COLOR))],
        ))
        .id()
}

#[must_use]
fn body(text: String, font: TextFont, commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                margin: UiRect::all(Val::Px(4.0)),
                padding: UiRect::all(Val::Px(4.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..Default::default()
            },
            BackgroundColor(POPUP_BODY_BACKGROUND_COLOR),
            BorderColor::all(POPUP_BODY_BORDER_COLOR),
            children![(Text(text), font)],
        ))
        .id()
}

fn root(
    child_entities: &[Entity],
    child_spawner: impl FnOnce(&mut RelatedSpawnerCommands<ChildOf>),
    commands: &mut Commands,
) -> Entity {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                margin: UiRect::AUTO,
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..Default::default()
            },
            BackgroundColor(POPUP_BACKGROUND_COLOR),
            BorderColor::all(POPUP_BORDER_COLOR),
            ZIndex(99),
        ))
        .add_children(child_entities)
        .with_children(child_spawner)
        .id()
}

pub(crate) fn spawn_popup(In(popup): In<Popup>, mut commands: Commands, assets: Res<AssetServer>) {
    let doto = assets.load::<Font>(URI_FONT_DOTO_BLACK);
    let body_font = assets.load::<Font>(URI_FONT_WDXL_LUBRIFONT_SC);

    let title_font = TextFont::from(doto.clone()).with_font_size(POPUP_TITLE_FONT_SIZE);
    let dismiss_font = TextFont::from(doto).with_font_size(POPUP_BUTTON_FONT_SIZE);
    let body_font = TextFont::from(body_font).with_font_size(POPUP_BODY_FONT_SIZE);

    let title = title(popup.title, title_font, &mut commands);
    let body = body(popup.body, body_font, &mut commands);
    let close_button = TextButtonBuilder {
        extra: Node {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            min_height: Val::Px(48.0),
            min_width: Val::Px(48.0),
            ..Default::default()
        },
        text_extra: (),
        text: fl!("popup__general__dismiss"),
        font: &dismiss_font,
        color: POPUP_DISMISS_COLOR,
        hover_color: POPUP_DISMISS_HOVER_COLOR,
        active_color: POPUP_DISMISS_ACTIVE_COLOR,
    }
    .build();

    root(
        &[title, body],
        |parent| {
            let target = parent.target_entity();
            parent.spawn(close_button).observe(
                move |_: On<Pointer<Click>>, mut commands: Commands| {
                    commands.entity(target).despawn();
                },
            );
        },
        &mut commands,
    );
}
