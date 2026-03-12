use core::{error::Error, fmt::Display};

use bevy::prelude::*;
use derive_more::{Deref, IsVariant};
use keplerian_sim::{Orbit2D, OrbitTrait2D};
use serde::{Deserialize, Serialize};

/// Marks this entity's relation with a parent celestial body.
#[derive(Clone, Copy, Component, Debug)]
#[require(RailMode)]
#[relationship(relationship_target = CelestialChildren)]
pub struct CelestialParent {
    #[relationship]
    pub entity: Entity,
}

#[derive(Component, Deref)]
#[relationship_target(relationship = CelestialParent, linked_spawn)]
pub struct CelestialChildren(Vec<Entity>);

impl CelestialChildren {
    #[must_use]
    pub(crate) fn clone_to_box(&self) -> Box<[Entity]> {
        Box::from(self.0.as_slice())
    }
}

/// How this entity behaves on-rails.
#[derive(Clone, Copy, Component, Debug, Default, PartialEq, IsVariant)]
pub enum RailMode {
    /// When on-rails, the object should stay static in terms of root-space
    /// coordinates.
    #[default]
    None,
    /// When on-rails, the object should follow a Keplerian orbit.
    Orbit(Orbit2D),
    /// This vessel should stay static relative to land.
    Surface(SurfaceAttachment),
}

impl RailMode {
    /// Gets the orbit in this rail, if any.
    #[must_use]
    pub const fn as_orbit(&self) -> Option<Orbit2D> {
        match self {
            Self::Orbit(o) => Some(*o),
            _ => None,
        }
    }

    /// Gets the surface attachment in this rail, if any.
    #[must_use]
    pub const fn as_attachment(&self) -> Option<SurfaceAttachment> {
        match self {
            Self::Surface(a) => Some(*a),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RailModeMismatch;

impl Display for RailModeMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RailModeMismatch")
    }
}

impl Error for RailModeMismatch {}

impl TryInto<Orbit2D> for RailMode {
    type Error = RailModeMismatch;

    fn try_into(self) -> std::result::Result<Orbit2D, Self::Error> {
        self.as_orbit().ok_or(RailModeMismatch)
    }
}

impl TryFrom<RailMode> for SurfaceAttachment {
    type Error = RailModeMismatch;

    fn try_from(value: RailMode) -> std::result::Result<Self, Self::Error> {
        value.as_attachment().ok_or(RailModeMismatch)
    }
}

impl Display for RailMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "== none =="),
            Self::Orbit(o) => write!(
                f,
                "=( e={:.3e}; p={:.8e}; ω={:.3}; M={:.5e}; μ={:.3e} )=",
                o.get_eccentricity(),
                o.get_periapsis(),
                o.get_arg_pe(),
                o.get_mean_anomaly_at_epoch(),
                o.get_gravitational_parameter(),
            ),
            Self::Surface(a) => write!(f, "=[ {:.8e} rad @ {:.5e} m ]=", a.angle, a.radius),
        }
    }
}

/// Denotes an attachment of a vessel relative to a body's surface.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfaceAttachment {
    /// The angle from the +x axis line that this
    /// vessel is landed on.
    pub angle: f64,
    /// How far away from the planetary core this vessel is landed on.
    pub radius: f64,
}
