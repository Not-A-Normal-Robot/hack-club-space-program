use bevy::prelude::*;

#[derive(Clone, Copy, Component)]
pub struct InactiveTextColor(pub Color);

#[derive(Clone, Copy, Component)]
pub struct HoverTextColor(pub Color);

#[derive(Clone, Copy, Component)]
pub struct ActiveTextColor(pub Color);
