//! Oribar: Orientation Bar
//!
//!
//! -360     -180       0       180      360
//!   | -b--x- | -q--o- | -b--x- | -q--o- |
//!
//! Degrees to percentage: theta / 720.0 + 0.5
//! Radians to percentage: theta / (4 * PI) + 0.5

use core::f64::consts::{FRAC_1_PI, FRAC_PI_2};

use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

use crate::{
    assets::fonts::URI_FONT_WDXL_LUBRIFONT_SC,
    checked_assign,
    components::main_game::{
        frames::RootSpacePosition,
        ui::oribar::{Oribar, OribarIndicator},
    },
    consts::{
        colors::{ORIBAR_BACKGROUND, scheme::ERROR},
        ui::oribar::{
            INDEX_TO_PERCENT, MarkIntensity, ORIBAR_HEIGHT, ORIBAR_INDICATOR_BOTTOM,
            ORIBAR_INDICATOR_HEIGHT, ORIBAR_INDICATOR_LEFT, ORIBAR_INDICATOR_WIDTH,
            ORIBAR_MARK_PER_REV, ORIBAR_NEEDLE_LEFT, ORIBAR_NEEDLE_WIDTH, get_oribar_vw,
        },
    },
    math::quat_to_rot,
    resources::{scene::GameScene, simulation::ActiveVessel},
};

fn create_mark(index: u16, commands: &mut Commands) -> Entity {
    // The percentage of the position at the middle of the line.
    let middle_percent = f32::from(index) * INDEX_TO_PERCENT;

    let intensity = MarkIntensity::from_index(index);
    let size = intensity.size();

    // The percentage of the position at the left of the line.
    let left_percent = size.x.mul_add(-0.5, middle_percent);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(size.x),
                height: Val::Percent(size.y),
                left: Val::Percent(left_percent),
                bottom: Val::ZERO,
                ..Default::default()
            },
            BackgroundColor(intensity.color()),
        ))
        .id()
}

/// `eighth` is assumed to range from 0 to 16, inclusively on both ends.
fn create_text(eighth: u16, font: &TextFont, commands: &mut Commands) -> Entity {
    let intensity = MarkIntensity::from_eighth(eighth);
    let size = intensity.size();
    let color = intensity.color();

    let left = f32::from(eighth).mul_add(const { 100.0 / 16.0 }, size.x.mul_add(0.5, 0.125));
    // let bottom = size.y - 50.0;
    let bottom = 5.0;

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(left),
                bottom: Val::Percent(bottom),
                ..Default::default()
            },
            font.clone(),
            Text((eighth.cast_signed() * -45).rem_euclid(360).to_string()),
            TextColor(color),
        ))
        .id()
}

pub(crate) fn init_oribar(
    screen: Single<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    let font = server.load::<Font>(URI_FONT_WDXL_LUBRIFONT_SC);
    let text_font = TextFont::from(font).with_font_size(20.0);

    let vw = get_oribar_vw(screen.size());

    let bundle = (
        Node {
            position_type: PositionType::Absolute,
            width: Val::Vw(vw),
            height: ORIBAR_HEIGHT,
            bottom: Val::ZERO,
            ..Default::default()
        },
        Oribar,
    );

    let rect = (
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(50.0),
            bottom: Val::ZERO,
            ..Default::default()
        },
        BackgroundColor(ORIBAR_BACKGROUND),
    );

    let mut children: Vec<Entity> =
        Vec::with_capacity(1 + (ORIBAR_MARK_PER_REV as usize * 2 + 1) + (16 + 1));

    children.push(commands.spawn(rect).id());

    children.extend((0..=ORIBAR_MARK_PER_REV * 2).map(|i| create_mark(i, &mut commands)));
    children.extend((0..=16).map(|i| create_text(i, &text_font, &mut commands)));

    commands.spawn(bundle).add_children(&children);

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: ORIBAR_NEEDLE_WIDTH,
            height: ORIBAR_HEIGHT,
            bottom: Val::ZERO,
            left: ORIBAR_NEEDLE_LEFT,
            ..Default::default()
        },
        BackgroundColor(ERROR),
        DespawnOnExit(GameScene::InGame),
        ZIndex(1),
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: ORIBAR_INDICATOR_WIDTH,
            height: ORIBAR_INDICATOR_HEIGHT,
            left: ORIBAR_INDICATOR_LEFT,
            bottom: ORIBAR_INDICATOR_BOTTOM,
            padding: UiRect::left(Val::Px(8.0)),
            ..Default::default()
        },
        BackgroundColor(ORIBAR_BACKGROUND),
        text_font,
        Text(String::new()),
        OribarIndicator,
    ));
}

pub(crate) fn update_oribar(
    mut oribar: Single<&mut Node, With<Oribar>>,
    mut indicator: Single<&mut Text, With<OribarIndicator>>,
    screen: Single<&Window, With<PrimaryWindow>>,
    tf_query: Query<&Transform>,
    parent_query: Query<&RootSpacePosition>,
    active_vessel: Res<ActiveVessel>,
) {
    /*
    1. Get relative pos of vessel to parent
    2. Get angle between rel vector to +x (atan2) to get longitude
    3. Root-space transform rotation - longitude
    4. Don't forget rem_euclid!
     */
    let Ok(transform) = tf_query.get(active_vessel.entity) else {
        return;
    };

    let Ok(parent_pos) = parent_query.get(active_vessel.prev_tick_parent) else {
        return;
    };

    let rel_pos = active_vessel.prev_tick_position.0 - parent_pos.0;
    let longitude = rel_pos.to_angle();

    let rootspace_rotation = quat_to_rot(transform.rotation);

    let rel_rotation = rootspace_rotation - longitude + FRAC_PI_2;

    // We need to shift it by +50vw
    let total_vw = f64::from(get_oribar_vw(screen.size()));

    let selected_vw = total_vw * rel_rotation * const { 0.25 * FRAC_1_PI };

    let left_vw = selected_vw + 50.0;
    let left_vw = left_vw.rem_euclid(total_vw / 2.0) - total_vw / 2.0;

    #[expect(clippy::cast_possible_truncation)]
    let val = Val::Vw(left_vw as f32);

    checked_assign!(oribar.left, val);

    let rel_rotation_str = format!("{:05.1}°", rel_rotation.to_degrees().rem_euclid(360.0));

    checked_assign!(indicator.0, rel_rotation_str);
}

pub(crate) fn handle_resize(
    mut oribar: Single<&mut Node, With<Oribar>>,
    screen: Single<&Window, With<PrimaryWindow>>,
    mut messages: MessageReader<WindowResized>,
) {
    if messages.is_empty() {
        return;
    }

    messages.clear();

    let vw = get_oribar_vw(screen.size());

    checked_assign!(oribar.width, Val::Vw(vw));
}
