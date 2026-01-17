use bevy::prelude::*;

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
