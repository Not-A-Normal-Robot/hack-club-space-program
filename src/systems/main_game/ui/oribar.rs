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
use strum::IntoEnumIterator;

use crate::{
    assets::fonts::URI_FONT_JETBRAINS_MONO,
    checked_assign,
    components::main_game::{
        frames::{RootSpaceLinearVelocity, RootSpacePosition},
        ui::oribar::{Oribar, OribarIndicator, OribarOverlay},
    },
    consts::{
        colors::{ORIBAR_BACKGROUND, scheme::ERROR, shades::TERTIARY_30},
        ui::oribar::{
            INDEX_TO_PERCENT, MarkIntensity, ORIBAR_CHILDREN_COUNT, ORIBAR_HEIGHT,
            ORIBAR_INDICATOR_BOTTOM, ORIBAR_INDICATOR_HEIGHT, ORIBAR_INDICATOR_LEFT,
            ORIBAR_INDICATOR_PADDING, ORIBAR_INDICATOR_WIDTH, ORIBAR_MARK_PER_REV,
            ORIBAR_NEEDLE_LEFT, ORIBAR_NEEDLE_WIDTH, get_oribar_vw,
        },
    },
    math::quat_to_rot,
    resources::{scene::GameScene, simulation::ActiveVessel},
};

/// Spawns a gradation mark with the specified index.
///
/// |""""|""""|""""|
#[must_use = "add this to a parent"]
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
            ZIndex(-2),
        ))
        .id()
}

/// `eighth` is assumed to range from 0 to 16, inclusively on both ends.
#[must_use = "add this to a parent"]
fn create_text(eighth: u16, font: &TextFont, commands: &mut Commands) -> Entity {
    let intensity = MarkIntensity::from_eighth(eighth);
    let size = intensity.size();
    let color = intensity.color();

    let left = f32::from(eighth).mul_add(const { 100.0 / 16.0 }, size.x.mul_add(0.5, 0.125));
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
            ZIndex(-1),
        ))
        .id()
}

/// Create an oribar overlay, e.g. prograde & retrograde.
#[must_use = "add this to a parent"]
fn create_overlay(overlay_kind: OribarOverlay, commands: &mut Commands) -> Entity {
    // Because of screen wrap we need 5 overlays:
    // + ... - ... + ... - ... +
    //             |

    let (pos_shape, neg_shape) = overlay_kind.get_icon_set();

    let wrapper = Node {
        position_type: PositionType::Absolute,
        top: Val::ZERO,
        left: Val::ZERO,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..Default::default()
    };

    let children: Box<[Entity]> = (0..=5u8)
        .map(|i| {
            let is_positive = i.is_multiple_of(2);

            let shape = if is_positive {
                pos_shape.clone()
            } else {
                neg_shape.clone()
            };

            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(25.0 * f32::from(i)),
                        bottom: Val::ZERO,
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        ..Default::default()
                    },
                    shape,
                ))
                .id()
        })
        .collect();

    commands.spawn(wrapper).add_children(&children).id()
}

pub(crate) fn init_oribar(
    screen: Single<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    let font = server.load::<Font>(URI_FONT_JETBRAINS_MONO);
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
        ZIndex(-3),
    );

    let mut children: Vec<Entity> = Vec::with_capacity(ORIBAR_CHILDREN_COUNT);

    children.push(commands.spawn(rect).id());

    children.extend((0..=ORIBAR_MARK_PER_REV * 2).map(|i| create_mark(i, &mut commands)));
    children.extend((0..=16).map(|i| create_text(i, &text_font, &mut commands)));
    children.extend(OribarOverlay::iter().map(|i| create_overlay(i, &mut commands)));

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
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            width: ORIBAR_INDICATOR_WIDTH,
            height: ORIBAR_INDICATOR_HEIGHT,
            left: ORIBAR_INDICATOR_LEFT,
            bottom: ORIBAR_INDICATOR_BOTTOM,
            padding: ORIBAR_INDICATOR_PADDING,
            ..Default::default()
        },
        BackgroundColor(ORIBAR_BACKGROUND),
        DespawnOnExit(GameScene::InGame),
        children![(OribarIndicator, Text(String::new()), text_font)],
    ));
}

#[derive(Component)]
pub(crate) struct OribarState {
    /// The rotation of the active vessel relative to its parent body.
    rel_rotation: f64,
    /// The prograde direction fo the active vessel relative to the
    /// parent body.
    vel_direction: f64,
    /// The size of the window in logical pixels.
    window_size: Vec2,
}

impl OribarState {
    #[must_use]
    fn new(
        tf_query: Query<&Transform>,
        sv_query: Query<(&RootSpacePosition, &RootSpaceLinearVelocity)>,
        active_vessel: Res<ActiveVessel>,
        screen: Single<&Window, With<PrimaryWindow>>,
    ) -> Option<Self> {
        let Ok(transform) = tf_query.get(active_vessel.entity) else {
            return None;
        };
        let root_rotation = quat_to_rot(transform.rotation);

        let Ok((parent_pos, parent_vel)) = sv_query.get(active_vessel.prev_tick_parent) else {
            return None;
        };

        let rel_pos = active_vessel.prev_tick_position.0 - parent_pos.0;
        let rel_vel = active_vessel.prev_tick_velocity.0 - parent_vel.0;

        let longitude = rel_pos.to_angle();
        let offset = -longitude + FRAC_PI_2;

        Some(Self {
            rel_rotation: root_rotation + offset,
            vel_direction: rel_vel.to_angle() + offset,
            window_size: screen.size(),
        })
    }

    fn update_oribar(&self, mut oribar: Single<&mut Node, With<Oribar>>) {
        // We need to shift it by +50vw
        let total_vw = f64::from(get_oribar_vw(self.window_size));

        let selected_vw = total_vw * self.rel_rotation * const { 0.25 * FRAC_1_PI };

        let left_vw = selected_vw + 50.0;
        let left_vw = left_vw.rem_euclid(total_vw / 2.0) - total_vw / 2.0;

        #[expect(clippy::cast_possible_truncation)]
        let val = Val::Vw(left_vw as f32);

        checked_assign!(oribar.left, val);
    }

    fn update_indicator(&self, mut indicator: Single<&mut Text, With<OribarIndicator>>) {
        let rel_rotation = self.rel_rotation.to_degrees().rem_euclid(360.0);
        let rel_rotation_str = format!("{rel_rotation:05.1}°");

        checked_assign!(indicator.0, rel_rotation_str);
    }
}

#[must_use]
pub(crate) fn calculate_oribar_state(
    screen: Single<&Window, With<PrimaryWindow>>,
    tf_query: Query<&Transform>,
    sv_query: Query<(&RootSpacePosition, &RootSpaceLinearVelocity)>,
    active_vessel: Res<ActiveVessel>,
) -> Option<OribarState> {
    OribarState::new(tf_query, sv_query, active_vessel, screen)
}

pub(crate) fn apply_oribar_state(
    In(state): In<Option<OribarState>>,
    oribar: Single<&mut Node, With<Oribar>>,
    indicator: Single<&mut Text, With<OribarIndicator>>,
) {
    let Some(state) = state else { return };

    state.update_oribar(oribar);
    state.update_indicator(indicator);
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
