use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

use crate::{
    assets::fonts::{
        URI_FONT_DOTO_BLACK, URI_FONT_DOTO_BOLD, URI_FONT_JETBRAINS_MONO,
        URI_FONT_JETBRAINS_MONO_ITALIC,
    },
    checked_assign,
    components::main_game::{
        celestial::{CelestialBody, Terrain},
        frames::RootSpacePosition,
        ui::altimeter::{
            Altimeter, AltimeterAltitudeText, AltimeterMobileAltitudeText,
            AltimeterMobileModeIndicator, AltimeterModeIndicator, AltimeterPrefix, AltimeterSign,
        },
    },
    consts::{
        colors::{
            ALTIMETER_ACTIVE, ALTIMETER_BACKGROUND, ALTIMETER_INACTIVE, ALTIMETER_INNER_BORDER,
            ALTIMETER_OUTER_BORDER, ALTIMETER_PREFIX,
        },
        ui::altimeter::{
            ALTIMETER_BIG_TEXT_SIZE, ALTIMETER_MEDIUM_TEXT_SIZE, ALTIMETER_MOBILE_CUTOFF,
            ALTIMETER_SMALL_TEXT_SIZE, ALTIMETER_TINY_TEXT_SIZE, AltitudeFormat, AltitudePrefix,
        },
    },
    fl,
    resources::{scene::GameScene, simulation::ActiveVessel, ui::AltimeterMode},
    systems::general::ui_activation::ActivationEvent,
    terrain::TerrainGen,
};

fn wrapper(children: &[Entity], commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                top: Val::ZERO,
                left: Val::ZERO,
                width: Val::Vw(100.0),
                justify_content: JustifyContent::End,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            DespawnOnExit(GameScene::InGame),
        ))
        .add_children(children)
        .id()
}

/// The container for the desktop altimeter.
#[must_use]
fn desktop_altimeter(children: &[Entity], window_width: f32, commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                display: if window_width > ALTIMETER_MOBILE_CUTOFF {
                    Display::Flex
                } else {
                    Display::None
                },
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(4.0)),
                border: UiRect::left(Val::Px(1.5)).with_bottom(Val::Px(1.5)),
                border_radius: BorderRadius::bottom_left(Val::Px(6.0)),
                ..Default::default()
            },
            BackgroundColor(ALTIMETER_BACKGROUND),
            BorderColor::all(ALTIMETER_OUTER_BORDER),
            Button,
            Altimeter { desktop_mode: true },
        ))
        .observe(
            |_: On<ActivationEvent>,
             mode: Res<State<AltimeterMode>>,
             mut next_mode: ResMut<NextState<AltimeterMode>>| {
                next_mode.set(mode.next());
            },
        )
        .add_children(children)
        .id()
}

/// The container for the mobile altimeter.
#[must_use]
fn mobile_altimeter(
    children: &[Entity],
    jetbrains_mono_italic: Handle<Font>,
    window_width: f32,
    commands: &mut Commands,
) -> Entity {
    let mut label_font =
        TextFont::from(jetbrains_mono_italic).with_font_size(ALTIMETER_TINY_TEXT_SIZE);
    label_font.weight = FontWeight(200);

    let label = commands
        .spawn((
            Text::new(fl!("altimeter__altitude__label")),
            label_font,
            TextColor(ALTIMETER_ACTIVE),
        ))
        .id();
    let inner = commands.spawn(Node::DEFAULT).add_children(children).id();
    commands
        .spawn((
            Node {
                display: if window_width <= ALTIMETER_MOBILE_CUTOFF {
                    Display::Flex
                } else {
                    Display::None
                },
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                min_width: Val::Px(48.0),
                min_height: Val::Px(48.0),
                ..Default::default()
            },
            BackgroundColor(ALTIMETER_BACKGROUND),
            Button,
            Altimeter {
                desktop_mode: false,
            },
        ))
        .observe(
            |_: On<ActivationEvent>,
             mode: Res<State<AltimeterMode>>,
             mut next_mode: ResMut<NextState<AltimeterMode>>| {
                next_mode.set(mode.next());
            },
        )
        .add_children(&[label, inner])
        .id()
}

/// The big text for the desktop altimeter.
#[must_use]
fn desktop_altitude(doto_bold: Handle<Font>, commands: &mut Commands) -> Entity {
    let doto_bold = TextFont::from(doto_bold.clone()).with_font_size(ALTIMETER_BIG_TEXT_SIZE);

    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
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
                    Text(AltitudePrefix::Meter.to_char().to_string()),
                    doto_bold,
                    TextColor(ALTIMETER_PREFIX),
                    AltimeterPrefix,
                ),
            ],
        ))
        .id()
}

#[must_use]
fn mobile_altitude(jetbrains_mono: Handle<Font>, commands: &mut Commands) -> Entity {
    let text_font = TextFont::from(jetbrains_mono).with_font_size(ALTIMETER_MEDIUM_TEXT_SIZE);

    commands
        .spawn((
            Node::DEFAULT,
            children![
                (
                    Text("0".into()),
                    AltimeterMobileAltitudeText,
                    TextColor(ALTIMETER_ACTIVE),
                    text_font.clone()
                ),
                (
                    Node {
                        margin: UiRect::left(Val::Px(2.0)).with_right(Val::Px(4.0)),
                        ..Default::default()
                    },
                    Text(AltitudePrefix::Meter.to_char().to_string()),
                    TextColor(ALTIMETER_PREFIX),
                    AltimeterPrefix,
                    text_font.clone()
                ),
                (
                    Text(AltimeterMode::AboveSeaLevel.to_string()),
                    AltimeterMobileModeIndicator,
                    TextColor(ALTIMETER_ACTIVE),
                    text_font.clone()
                )
            ],
        ))
        .id()
}

/// The display for the reference frame, for the desktop altimeter.
#[must_use]
fn ref_frame_display(font: TextFont, mode: AltimeterMode) -> impl Bundle {
    (
        Text(mode.to_string()),
        font,
        TextColor(ALTIMETER_ACTIVE),
        AltimeterModeIndicator(mode),
    )
}

/// The separator between reference frames, for the desktop altimeter.
#[must_use]
fn ref_frame_separator() -> impl Bundle {
    (
        Node {
            width: Val::Px(4.0),
            height: Val::Px(4.0),
            border_radius: BorderRadius::MAX,
            margin: UiRect::horizontal(Val::Px(8.0)),
            ..Default::default()
        },
        BackgroundColor(ALTIMETER_INNER_BORDER),
    )
}

/// The list of reference frames, for the desktop altimeter.
#[must_use]
fn ref_frame_displays(doto_black: Handle<Font>, commands: &mut Commands) -> Entity {
    let doto_black = TextFont::from(doto_black.clone()).with_font_size(ALTIMETER_SMALL_TEXT_SIZE);

    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..Default::default()
            },
            BorderColor::all(ALTIMETER_INNER_BORDER),
            children![
                ref_frame_display(doto_black.clone(), AltimeterMode::AboveSeaLevel),
                ref_frame_separator(),
                ref_frame_display(doto_black.clone(), AltimeterMode::AboveGroundLevel),
                ref_frame_separator(),
                ref_frame_display(doto_black.clone(), AltimeterMode::FromCentre),
            ],
        ))
        .id()
}

pub(crate) fn init_altimeter(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    server: Res<AssetServer>,
) {
    let doto_bold = server.load::<Font>(URI_FONT_DOTO_BOLD);
    let doto_black = server.load::<Font>(URI_FONT_DOTO_BLACK);
    let jetbrains_mono_italic = server.load::<Font>(URI_FONT_JETBRAINS_MONO_ITALIC);
    let jetbrains_mono = server.load::<Font>(URI_FONT_JETBRAINS_MONO);

    let window_width = window.width();

    let altitude = desktop_altitude(doto_bold, &mut commands);
    let ref_frames = ref_frame_displays(doto_black, &mut commands);
    let desktop = desktop_altimeter(&[altitude, ref_frames], window_width, &mut commands);

    let altitude = mobile_altitude(jetbrains_mono, &mut commands);
    let mobile = mobile_altimeter(
        &[altitude],
        jetbrains_mono_italic,
        window_width,
        &mut commands,
    );

    wrapper(&[desktop, mobile], &mut commands);
}

pub(crate) fn calculate_altitude_format(
    cel_query: Query<(&CelestialBody, &RootSpacePosition, &Terrain)>,
    active_vessel: Res<ActiveVessel>,
    altimeter_mode: Res<State<AltimeterMode>>,
) -> Option<AltitudeFormat> {
    let Ok((body, body_pos, terrain)) = cel_query.get(active_vessel.prev_tick_parent) else {
        return None;
    };

    let rel_pos = active_vessel.prev_tick_position.0 - body_pos.0;
    let dist = rel_pos.length();

    let altitude = match altimeter_mode.get() {
        AltimeterMode::FromCentre => dist,
        AltimeterMode::AboveSeaLevel => dist - f64::from(body.base_radius),
        AltimeterMode::AboveGroundLevel => {
            // TODO: Consider celestial rotation
            let theta = rel_pos.to_angle();
            let terrain = TerrainGen::new(*terrain);
            let terrain_altitude = terrain.get_terrain_altitude(theta);

            dist - terrain_altitude
        }
    };

    Some(AltitudeFormat::new(altitude))
}

type AltimeterSet<'w, 's, 'qw, 'qs> = ParamSet<
    'w,
    's,
    (
        Query<'qw, 'qs, &'static mut TextColor, With<AltimeterSign>>,
        Query<'qw, 'qs, &'static mut Text, With<AltimeterAltitudeText>>,
        Query<'qw, 'qs, &'static mut Text, With<AltimeterPrefix>>,
        Query<'qw, 'qs, &'static mut Text, With<AltimeterMobileAltitudeText>>,
    ),
>;

pub(crate) fn apply_altimeter_format(
    In(format): In<Option<AltitudeFormat>>,
    mut param_set: AltimeterSet,
) {
    let Some(format) = format else { return };

    for mut sign_color in param_set.p0() {
        sign_color.0 = if format.is_negative {
            ALTIMETER_ACTIVE
        } else {
            ALTIMETER_INACTIVE
        };
    }

    for mut altitude_text in param_set.p1() {
        altitude_text.0.clear();

        for char in format.desktop_numeric {
            altitude_text.0.push(char);
        }
    }

    for mut prefix_text in param_set.p2() {
        prefix_text.0.clear();
        prefix_text.0.push(format.prefix.to_char());
    }

    for mut mobile_text in param_set.p3() {
        mobile_text.0.clear();

        for char in format.mobile_numeric {
            mobile_text.0.push(char);
        }
    }
}

pub(crate) fn update_altimeter_ref_disp(
    desktop: Query<(&mut TextColor, &AltimeterModeIndicator)>,
    mobile: Query<&mut Text, With<AltimeterMobileModeIndicator>>,
    mode: Res<State<AltimeterMode>>,
) {
    let mode = mode.get();

    for (mut color, indicator) in desktop {
        let active = indicator.0 == *mode;

        color.0 = if active {
            ALTIMETER_ACTIVE
        } else {
            ALTIMETER_INACTIVE
        };
    }

    let stringified = mode.to_string();
    for mut text in mobile {
        checked_assign!(text.0, stringified, stringified.clone());
        text.0 = mode.to_string();
    }
}

pub(crate) fn handle_resize(
    query: Query<(&mut Node, &Altimeter)>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut messages: MessageReader<WindowResized>,
) {
    if messages.is_empty() {
        return;
    }

    messages.clear();

    let size = window.width();
    let desktop_mode = size > ALTIMETER_MOBILE_CUTOFF;

    for (mut node, altimeter) in query {
        node.display = if altimeter.desktop_mode == desktop_mode {
            Display::DEFAULT
        } else {
            Display::None
        };
    }
}
