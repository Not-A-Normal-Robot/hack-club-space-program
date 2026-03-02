//! Oribar: Orientation Bar
//!
//!
//! -360     -180       0       180      360
//!   | -b--x- | -q--o- | -b--x- | -q--o- |
//!
//! Degrees to percentage: theta / 720.0 + 0.5
//! Radians to percentage: theta / (4 * PI) + 0.5

use core::f32::consts::PI;

use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

use crate::{
    checked_assign,
    components::main_game::ui::oribar::Oribar,
    consts::{
        colors::ORIBAR_BACKGROUND,
        ui::oribar::{INDEX_TO_PERCENT, MarkIntensity, ORIBAR_MARK_PER_REV, get_oribar_vw},
    },
    math::quat_to_rot,
    resources::simulation::ActiveVessel,
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

pub(crate) fn init_oribar(screen: Single<&Window, With<PrimaryWindow>>, mut commands: Commands) {
    let vw = get_oribar_vw(screen.size());

    let bundle = (
        Node {
            position_type: PositionType::Absolute,
            display: Display::Flex,
            width: Val::Vw(vw),
            height: Val::Vh(6.0),
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
    let rect = commands.spawn(rect).id();

    let marks = (0..=ORIBAR_MARK_PER_REV * 2).map(|i| create_mark(i, &mut commands));

    // TODO: Texts i.e. "NESW"

    let children: Box<[Entity]> = core::iter::once(rect).chain(marks).collect();

    commands.spawn(bundle).add_children(&children);
}

pub(crate) fn update_oribar(
    mut oribar: Single<&mut Node, With<Oribar>>,
    screen: Single<&Window, With<PrimaryWindow>>,
    query: Query<&Transform>,
    active_vessel: Res<ActiveVessel>,
) {
    if let Ok(transform) = query.get(active_vessel.entity) {
        // TODO: Consider rotations relative to planet center (not world-space)

        #[expect(clippy::cast_possible_truncation)]
        let rotation = quat_to_rot(transform.rotation) as f32;

        // We need to shift it by +50vw
        let total_vw = get_oribar_vw(screen.size());

        let selected_vw = total_vw * rotation / (4.0 * PI);

        let left_vw = selected_vw + 50.0;

        let left_percent = left_vw.rem_euclid(total_vw / 2.0) - total_vw / 2.0;

        checked_assign!(oribar.left, Val::Vw(left_percent));
    }
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
