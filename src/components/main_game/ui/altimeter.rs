use crate::resources::ui::AltimeterMode;
use bevy::prelude::*;

/// The sign (+/-) of the altitude.
#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) struct AltimeterSign;

/// The multi-digit altitude text.
#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) struct AltimeterAltitudeText;

/// The altimeter's SI prefix or "m" (meter).
#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) struct AltimeterPrefix;

/// A reference frame indicator in the altimeter.
#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) struct AltimeterModeIndicator(pub(crate) AltimeterMode);
