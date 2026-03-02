//! Oribar: Orientation Bar
//!
//!
//! -360     -180       0       180      360
//!   | -b--x- | -q--o- | -b--x- | -q--o- |
//!
//! Degrees to percentage: theta / 720.0 + 0.5
//! Radians to percentage: theta / (4 * PI) + 0.5

use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

use crate::{
    checked_assign,
    components::main_game::ui::oribar::Oribar,
    consts::{
        colors::{
            ORIBAR_BACKGROUND,
            scheme::{PRIMARY, SECONDARY, TERTIARY},
            shades::{NEUTRAL_50, NEUTRAL_70, NEUTRAL_90},
        },
        ui::{ORIBAR_MARK_PER_REV, get_oribar_vw},
    },
    resources::simulation::ActiveVessel,
};

/// Categories for the intensities of marks.
/// The higher the discriminant, the lesser the intensity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MarkIntensity {
    /// -360°, 0°, or +360°
    ///
    /// At indices that are a multiple of [`ORIBAR_MARK_PER_REV`].
    North,
    /// 180°, +180°
    ///
    /// At indices that are a multiple of [`ORIBAR_MARK_PER_REV`] / 2,
    /// but not [`ORIBAR_MARK_PER_REV`] itself.
    South,
    /// -270°, -90°, +90°, +270°
    ///
    /// At indices that are a multiple of [`ORIBAR_MARK_PER_REV`] / 4,
    /// but not [`ORIBAR_MARK_PER_REV`] / 2.
    Horizontal,
    /// -315°, -225°, -135°, -45°, +45°, +135°, +225°, +315°
    ///
    /// At indices that are a multiple of [`ORIBAR_MARK_PER_REV`] / 8,
    /// but not [`ORIBAR_MARK_PER_REV`] / 4.
    Eighths,
    /// At even indices.
    Even,
    /// At odd indices.
    Odd,
}

impl MarkIntensity {
    const fn from_index(index: u16) -> Self {
        if index.is_multiple_of(ORIBAR_MARK_PER_REV) {
            Self::North
        } else if index.is_multiple_of(ORIBAR_MARK_PER_REV / 2) {
            Self::South
        } else if index.is_multiple_of(ORIBAR_MARK_PER_REV / 4) {
            Self::Horizontal
        } else if index.is_multiple_of(ORIBAR_MARK_PER_REV / 8) {
            Self::Eighths
        } else if index.is_multiple_of(2) {
            Self::Even
        } else {
            Self::Odd
        }
    }

    /// Returns the width in percentage of the mark at this intensity.
    const fn width(self) -> f32 {
        match self {
            Self::North | Self::South | Self::Horizontal => 0.15,
            Self::Eighths => 0.125,
            Self::Even | Self::Odd => 0.1,
        }
    }

    /// Returns the height in percentage of the mark at this intensity.
    const fn height(self) -> f32 {
        match self {
            Self::North => 100.0,
            Self::South => 87.5,
            Self::Horizontal => 75.0,
            Self::Eighths => 50.0,
            Self::Even => 30.0,
            Self::Odd => 25.0,
        }
    }

    /// Returns the size in percentage of the mark at this intensity.
    const fn size(self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    /// Returns the color of the mark at this intensity.
    const fn color(self) -> Color {
        match self {
            Self::North => PRIMARY,
            Self::South => SECONDARY,
            Self::Horizontal => TERTIARY,
            Self::Eighths => NEUTRAL_90,
            Self::Even | Self::Odd => NEUTRAL_70,
        }
    }
}

const INDEX_TO_PERCENT: f32 = 50.0 / ORIBAR_MARK_PER_REV as f32;

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

    let children: Box<[Entity]> = core::iter::once(rect).chain(marks).collect();

    commands.spawn(bundle).add_children(&children);
}

pub(crate) fn update_oribar(
    mut oribar: Single<&mut Node, With<Oribar>>,
    active_vessel: Res<ActiveVessel>,
) {
    // TODO: Save active vessel prev orientation
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
