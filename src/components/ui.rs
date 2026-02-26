use bevy::prelude::*;

/// The text color for when this entity (or its direct child)
/// is not being hovered upon.
#[derive(Clone, Copy, Component, Default)]
pub struct InactiveTextColor(pub Color);

/// The text color for when this entity (or its direct child)
/// is being hovered upon.
#[derive(Clone, Copy, Component)]
#[require(InactiveTextColor)]
pub struct HoverTextColor(pub Color);

/// The text color for when this entity (or its direct child)
/// is actively being pressed.
#[derive(Clone, Copy, Component)]
#[require(InactiveTextColor)]
pub struct ActiveTextColor(pub Color);
