use bevy::prelude::*;
use keplerian_sim::Orbit2D;

/// Marks this entity's relation with a parent celestial body.
#[derive(Clone, Copy, Component, Debug)]
#[require(RailMode)]
#[relationship(relationship_target = CelestialChild)]
pub struct CelestialParent {
    #[relationship]
    pub entity: Entity,
}

#[derive(Component)]
#[relationship_target(relationship = CelestialParent, linked_spawn)]
pub struct CelestialChild(Vec<Entity>);

/// How this entity behaves on-rails.
#[derive(Clone, Component, Debug, Default, PartialEq)]
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
    pub fn as_orbit(&self) -> Option<Orbit2D> {
        match self {
            Self::Orbit(o) => Some(o.clone()),
            _ => None,
        }
    }

    /// Gets the surface attachment in this rail, if any.
    pub fn as_attachment(&self) -> Option<SurfaceAttachment> {
        match self {
            Self::Surface(a) => Some(*a),
            _ => None,
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
