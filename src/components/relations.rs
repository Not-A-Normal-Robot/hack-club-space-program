use core::fmt::Display;

use bevy::prelude::*;
use keplerian_sim::{Orbit2D, OrbitTrait2D};

/// Marks this entity's relation with a parent celestial body.
#[derive(Clone, Copy, Component, Debug)]
#[require(RailMode)]
#[relationship(relationship_target = CelestialChildren)]
pub struct CelestialParent {
    #[relationship]
    pub entity: Entity,
}

#[derive(Component)]
#[relationship_target(relationship = CelestialParent, linked_spawn)]
pub struct CelestialChildren(Vec<Entity>);

impl CelestialChildren {
    #[must_use] 
    pub fn clone_to_box(&self) -> Box<[Entity]> {
        Box::from(self.0.as_slice())
    }
}

/// How this entity behaves on-rails.
#[derive(Clone, Copy, Component, Debug, Default, PartialEq)]
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
    /// Checks whether or not this is the [`None`][RailMode::None] variant.
    #[must_use] 
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Checks whether or not this is the [`Orbit`][RailMode::Orbit] variant.
    #[must_use] 
    pub const fn is_orbit(&self) -> bool {
        matches!(self, Self::Orbit(_))
    }

    /// Checks whether or not this is the [`Surface`][RailMode::Surface] variant.
    #[must_use] 
    pub const fn is_surface(&self) -> bool {
        matches!(self, Self::Surface(_))
    }

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

pub struct RailModeMismatch;

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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceAttachment {
    /// The angle from the +x axis line that this
    /// vessel is landed on.
    pub angle: f64,
    /// How far away from the planetary core this vessel is landed on.
    pub radius: f64,
}
