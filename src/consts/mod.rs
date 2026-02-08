use crate::components::{celestial::CelestialBody, vessel::Vessel};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub mod keybinds;

/// The gravitational constant, in m^3 kg^-1 s^-2.
pub const GRAVITATIONAL_CONSTANT: f64 = 6.6743e-11;

pub type FilterLoadedVessels = (
    With<Vessel>,
    Without<RigidBodyDisabled>,
    Without<CelestialBody>,
);

pub type FilterUnloadedVessels = (
    With<Vessel>,
    With<RigidBodyDisabled>,
    Without<CelestialBody>,
);
