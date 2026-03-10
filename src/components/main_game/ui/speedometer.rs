use bevy::prelude::*;

/// The part of the speedometer that shows the current speed
/// relative to the parent body.
#[derive(Clone, Copy, Component)]
pub(crate) struct TotalSpeedometerText;

/// The part of the speedometer that shows the current tangential
/// speed relative to the parent body.
#[derive(Clone, Copy, Component)]
pub(crate) struct HorizontalSpeedometerText;

/// The part of the speedometer that shows the current normal
/// speed relative to the parent body.
#[derive(Clone, Copy, Component)]
pub(crate) struct VerticalSpeedometerText;

/// The part of the speedometer that shows the current speed unit,
/// e.g., m/s
#[derive(Clone, Copy, Component)]
pub(crate) struct SpeedometerUnitText;
