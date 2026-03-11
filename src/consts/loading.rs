use bevy::prelude::*;
use keplerian_sim::{CompactOrbit2D, Orbit2D};

use crate::consts::GRAVITATIONAL_CONSTANT;

/// Holds static information about celestial bodies.
///
/// Should be in sync with `./save_data.schema.json`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CelestialData {
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // ================== MEMO ==================
    //  DO NOT FORGET TO UPDATE THE SCHEMA FILE
    //        AT `./save_data.schema.json`
    //         IF YOU ARE EDITING THIS!
    // ==========================================
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    /// The name of this celestial body.
    pub name: &'static str,
    /// The mass of this celestial body, in kilograms.
    pub mass: f64,
    /// The radius of this celestial body, in metres.
    pub radius: f64,
    /// The color of this celestial body.
    pub color: Color,
    /// Information about this celestial body's orbital
    /// parameters, if any.
    pub orbit: Option<CelestialOrbitalData>,
    /// A list of this celestial body's children.
    pub children: &'static [&'static CelestialData],
}

/// Holds static information about celestial bodies' orbits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CelestialOrbitalData {
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // ================== MEMO ==================
    //  DO NOT FORGET TO UPDATE THE SCHEMA FILE
    //        AT `./save_data.schema.json`
    //         IF YOU ARE EDITING THIS!
    // ==========================================
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    /// The ecccentricity of this celestial body's
    /// orbit.
    pub eccentricity: f64,
    /// The periapsis of this celestial body's orbit,
    /// in metres.
    pub periapsis: f64,
    /// The argument of periapsis of this celestial
    /// body's orbit, in radians.
    pub arg_pe: f64,
    /// The mean anomaly at epoch of this celestial
    /// body's orbit, in radians.
    pub mean_anomaly: f64,
}

impl CelestialOrbitalData {
    #[inline]
    #[must_use]
    pub const fn to_compact_orbit(self, parent_mass: f64) -> CompactOrbit2D {
        CompactOrbit2D {
            eccentricity: self.eccentricity,
            periapsis: self.eccentricity,
            arg_pe: self.arg_pe,
            mean_anomaly: self.mean_anomaly,
            mu: parent_mass * GRAVITATIONAL_CONSTANT,
        }
    }

    #[inline]
    #[must_use]
    pub fn to_cached_orbit(self, parent_mass: f64) -> Orbit2D {
        self.to_compact_orbit(parent_mass).into()
    }
}
