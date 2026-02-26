use bevy::prelude::*;
use core::ops::Range;

use crate::terrain::TerrainPoint;

#[derive(Clone, Component, Debug, PartialEq, Eq)]
pub(crate) struct PrevIndexRanges(pub(crate) Box<[Range<u32>]>);

#[derive(Clone, Component, Debug, PartialEq)]
pub(crate) struct PrevColliderPoints(pub(crate) Vec<TerrainPoint>);
