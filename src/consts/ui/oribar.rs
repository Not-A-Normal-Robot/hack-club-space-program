use crate::consts::colors::{
    scheme::{PRIMARY, SECONDARY, TERTIARY},
    shades::{NEUTRAL_70, NEUTRAL_90},
};
use bevy::prelude::*;

/// The threshold of when to switch oribar scales.
///
/// The first entry that contains a value greater
/// than the logical width of the screen denotes
/// which span index to use.
///
/// If the screen's logical width is lesser than all
/// of the thresholds, then a default of [`ORIBAR_MAX_ZOOM`]
/// shall be used.
pub(crate) const ORIBAR_THRESHOLDS: [f32; 6] = [1200.0, 800.0, 600.0, 400.0, 300.0, 200.0];
pub(crate) const ORIBAR_VW: [f32; 6] = [
    // 360° per 100vw
    // We need 720° to account for wraparound
    720.0 / 360.0 * 100.0,
    // 270° per 100vw
    720.0 / 270.0 * 100.0,
    // 180° per 100vw
    720.0 / 180.0 * 100.0,
    // 135° per 100vw
    720.0 / 135.0 * 100.0,
    // 90° per 100vw
    720.0 / 90.0 * 100.0,
    // 60° per 100vw
    720.0 / 60.0 * 100.0,
];
pub(crate) const ORIBAR_MAX_ZOOM: f32 = 720.0 / 45.0 * 100.0;

const _: () = assert!(
    ORIBAR_THRESHOLDS.len() == ORIBAR_VW.len(),
    "Oribar threshold and vw array sizes must match"
);

/// The amount of marks an oribar has every revolution.
/// In total, an oribar contains twice this many amount of marks.
pub(crate) const ORIBAR_MARK_PER_REV: u16 = 72;

const _: () = {
    assert!(
        ORIBAR_MARK_PER_REV.checked_mul(2).is_some(),
        "ORIBAR_MARK_PER_REV must be less than half of u16::MAX"
    );
    assert!(
        ORIBAR_MARK_PER_REV.is_multiple_of(8),
        "ORIBAR_MARK_PER_REV must be a multiple of 8 for NS/SE/SW/NW marks to appear"
    );
};

/// Gets the screen size of the full 720° oribar in vw, based on
/// the screen resolution.
pub(crate) fn get_oribar_vw(screen_res: Vec2) -> f32 {
    let x = screen_res.x;

    ORIBAR_THRESHOLDS
        .into_iter()
        .enumerate()
        .find_map(|(index, threshold)| (x > threshold).then_some(ORIBAR_VW[index]))
        .unwrap_or(ORIBAR_MAX_ZOOM)
}

/// Categories for the intensities of marks.
/// The higher the discriminant, the lesser the intensity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum MarkIntensity {
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
    pub(crate) const fn from_index(index: u16) -> Self {
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

    pub(crate) const fn from_eighth(index: u16) -> Self {
        if index.is_multiple_of(8) {
            Self::North
        } else if index.is_multiple_of(4) {
            Self::South
        } else if index.is_multiple_of(2) {
            Self::Horizontal
        } else {
            Self::Eighths
        }
    }

    /// Returns the width in percentage of the mark at this intensity.
    pub(crate) const fn width(self) -> f32 {
        match self {
            Self::North | Self::South | Self::Horizontal => 0.15,
            Self::Eighths => 0.125,
            Self::Even | Self::Odd => 0.1,
        }
    }

    /// Returns the height in percentage of the mark at this intensity.
    pub(crate) const fn height(self) -> f32 {
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
    pub(crate) const fn size(self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    /// Returns the color of the mark at this intensity.
    pub(crate) const fn color(self) -> Color {
        match self {
            Self::North => PRIMARY,
            Self::South => SECONDARY,
            Self::Horizontal => TERTIARY,
            Self::Eighths => NEUTRAL_90,
            Self::Even | Self::Odd => NEUTRAL_70,
        }
    }
}

pub(crate) const INDEX_TO_PERCENT: f32 = 50.0 / ORIBAR_MARK_PER_REV as f32;

pub(crate) const ORIBAR_HEIGHT_PX: f32 = 64.0;
pub(crate) const ORIBAR_HEIGHT: Val = Val::Px(ORIBAR_HEIGHT_PX);

pub(crate) const ORIBAR_NEEDLE_WIDTH_VW: f32 = 0.25;
pub(crate) const ORIBAR_NEEDLE_WIDTH: Val = Val::Vw(ORIBAR_NEEDLE_WIDTH_VW);

pub(crate) const ORIBAR_NEEDLE_LEFT_VW: f32 = 50.0 - (ORIBAR_NEEDLE_WIDTH_VW / 2.0);
pub(crate) const ORIBAR_NEEDLE_LEFT: Val = Val::Vw(ORIBAR_NEEDLE_LEFT_VW);

pub(crate) const ORIBAR_INDICATOR_LEFT_VW: f32 = 50.0 + (ORIBAR_NEEDLE_WIDTH_VW / 2.0);
pub(crate) const ORIBAR_INDICATOR_LEFT: Val = Val::Vw(ORIBAR_INDICATOR_LEFT_VW);

pub(crate) const ORIBAR_INDICATOR_WIDTH: Val = Val::Px(84.0);
pub(crate) const ORIBAR_INDICATOR_HEIGHT: Val = Val::Px(ORIBAR_HEIGHT_PX / 2.0);
pub(crate) const ORIBAR_INDICATOR_BOTTOM: Val = Val::Px(ORIBAR_HEIGHT_PX / 2.0);
pub(crate) const ORIBAR_INDICATOR_PADDING: UiRect = UiRect::axes(Val::Px(8.0), Val::Px(4.0));
