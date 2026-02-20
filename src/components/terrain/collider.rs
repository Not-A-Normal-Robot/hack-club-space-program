use bevy::prelude::*;
use core::ops::Range;

use crate::terrain::TerrainPoint;

#[derive(Clone, Component, Debug, PartialEq, Eq)]
pub struct PrevIndexRanges(pub Box<[Range<u32>]>);

#[derive(Clone, Component, Debug, PartialEq)]
pub struct PrevColliderPoints(pub Vec<TerrainPoint>);
