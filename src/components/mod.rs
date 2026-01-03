use bevy::prelude::*;

pub mod frames;

#[derive(Component)]
pub struct ParentBody(pub Entity);

#[derive(Component)]
pub struct Heightmap(pub Box<[f64]>);

#[derive(Component)]
pub struct Body;
