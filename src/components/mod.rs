use bevy::{math::DVec2, prelude::*};

pub mod frames;

#[derive(Clone, Copy, Component)]
#[relationship(relationship_target = ChildObjects)]
pub struct ParentBody(pub Entity);

#[derive(Component)]
pub struct ChildObjects(Vec<Entity>);

impl RelationshipTarget for ChildObjects {
    const LINKED_SPAWN: bool = true;

    type Relationship = ParentBody;

    type Collection = Vec<Entity>;

    fn collection(&self) -> &Self::Collection {
        &self.0
    }

    fn collection_mut_risky(&mut self) -> &mut Self::Collection {
        &mut self.0
    }

    fn from_collection_risky(collection: Self::Collection) -> Self {
        Self(collection)
    }
}

#[derive(Clone, Component, Default)]
pub struct Heightmap(pub Box<[f32]>);

#[derive(Clone, Copy, Component)]
#[require(Heightmap)]
pub struct CelestialBody {
    pub radius: f32,
}

#[derive(Clone, Copy, Component)]
pub struct Vessel;

#[derive(Clone, Copy, Component)]
pub struct SimCameraTransform {
    pub translation: DVec2,
    pub zoom: f64,
}

#[derive(Clone, Copy, Component)]
pub struct SimCamera;
