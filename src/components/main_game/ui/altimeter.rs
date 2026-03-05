use crate::resources::ui::AltimeterMode;
use bevy::prelude::*;

/// The altimeter's root element.
#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) struct Altimeter {
    /// Whether or not this altimeter is meant for desktop mode.
    pub(crate) desktop_mode: bool,
}

/// The sign (+/-) of the altitude, for the desktop altimeter.
#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) struct AltimeterSign;

/// The multi-digit altitude text, for the desktop altimeter.
#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) struct AltimeterAltitudeText;

/// The altitude text, for the mobile altimeter.
#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) struct AltimeterMobileAltitudeText;

/// The altimeter's SI prefix or "m" (meter), for both desktop and mobile.
#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) struct AltimeterPrefix;

/// A reference frame indicator in the altimeter, for the desktop altimeter.
#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) struct AltimeterModeIndicator(pub(crate) AltimeterMode);

/// The reference frame indicator in the mobile altimeter.
#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) struct AltimeterMobileModeIndicator;
