use bevy::prelude::*;

use crate::{
    assets::fonts::{URI_FONT_DOTO_BLACK, URI_FONT_DOTO_BOLD},
    components::main_game::{
        celestial::{CelestialBody, Terrain},
        frames::RootSpacePosition,
        ui::altimeter::{
            AltimeterAltitudeText, AltimeterModeIndicator, AltimeterPrefix, AltimeterSign,
        },
    },
    consts::{
        colors::{
            ALTIMETER_ACTIVE, ALTIMETER_BACKGROUND, ALTIMETER_INACTIVE, ALTIMETER_INNER_BORDER,
            ALTIMETER_OUTER_BORDER, ALTIMETER_PREFIX,
        },
        ui::altimeter::{
            ALTIMETER_BIG_TEXT_SIZE, ALTIMETER_SMALL_TEXT_SIZE, AltimeterState, AltitudeFormat,
        },
    },
    fl,
    resources::{scene::GameScene, simulation::ActiveVessel, ui::AltimeterMode},
    terrain::TerrainGen,
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
fn altitude(doto_bold: Handle<Font>, commands: &mut Commands) -> Entity {
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

fn ref_frame_display(font: TextFont, mode: AltimeterMode) -> impl Bundle {
    let text = match mode {
        AltimeterMode::AboveGroundLevel => fl!("altimeter__mode__agl__text"),
        AltimeterMode::AboveSeaLevel => fl!("altimeter__mode__asl__text"),
        AltimeterMode::FromCentre => fl!("altimeter__mode__ctr__text"),
    };

    (
        Text(text),
        font,
        TextColor(ALTIMETER_ACTIVE),
        AltimeterModeIndicator(mode),
    )
}

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

fn ref_frame_displays(doto_black: Handle<Font>, commands: &mut Commands) -> Entity {
    let doto_black = TextFont::from(doto_black.clone()).with_font_size(ALTIMETER_SMALL_TEXT_SIZE);

    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                border: UiRect::all(Val::Px(2.0)),
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

pub(crate) fn init_altimeter(mut commands: Commands, server: Res<AssetServer>) {
    let doto_bold = server.load::<Font>(URI_FONT_DOTO_BOLD);
    let doto_black = server.load::<Font>(URI_FONT_DOTO_BLACK);

    let altitude = altitude(doto_bold, &mut commands);
    let ref_frames = ref_frame_displays(doto_black, &mut commands);
    root(&[altitude, ref_frames], &mut commands);
}

pub(crate) fn calculate_altimeter_state(
    cel_query: Query<(&CelestialBody, &RootSpacePosition, &Terrain)>,
    active_vessel: Res<ActiveVessel>,
    altimeter_mode: Res<State<AltimeterMode>>,
) -> Option<AltimeterState> {
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

    Some(AltimeterState {
        format: AltitudeFormat::new(altitude),
        mode: *altimeter_mode.get(),
    })
}

pub(crate) fn apply_altimeter_state(In(state): In<Option<AltimeterState>>) {
    let Some(state) = state else { return };
    dbg!(state);
    warn!("Altimeter state application is not implemented yet");
    // todo!();
}
