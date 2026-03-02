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
