use std::sync::LazyLock;

use crate::components::{celestial::CelestialBody, vessel::Vessel};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};

pub(crate) mod colors;
pub(crate) mod controls;
pub(crate) mod terrain;

/// The gravitational constant, in m^3 kg^-1 s^-2.
pub const GRAVITATIONAL_CONSTANT: f64 = 6.6743e-11;

/// The selector for the canvas in the WASM version of this game.
pub(crate) const WEB_CANVAS_SELECTOR: &str = "#h";

pub(crate) type FilterLoadedVessels = (
    With<Vessel>,
    Without<RigidBodyDisabled>,
    Without<CelestialBody>,
);

pub(crate) type FilterUnloadedVessels = (
    With<Vessel>,
    With<RigidBodyDisabled>,
    Without<CelestialBody>,
);

pub(crate) const TAB_FOCUS_OUTLINE: Outline = Outline {
    color: Color::WHITE,
    width: Val::Px(2.0),
    offset: Val::Px(2.0),
};

pub(crate) static FLUENT_LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> =
    LazyLock::new(|| fluent_language_loader!());
