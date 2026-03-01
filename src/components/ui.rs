use bevy::prelude::*;

/// The text color for when this entity (or its direct child)
/// is not being hovered upon.
#[derive(Clone, Copy, Component, Default, Debug, PartialEq)]
pub(crate) struct InactiveTextColor(pub(crate) Color);

/// The text color for when this entity (or its direct child)
/// is being hovered upon.
#[derive(Clone, Copy, Component, Debug, PartialEq)]
#[require(InactiveTextColor)]
pub(crate) struct HoverTextColor(pub(crate) Color);

/// The text color for when this entity (or its direct child)
/// is actively being pressed.
#[derive(Clone, Copy, Component, Debug, PartialEq)]
#[require(InactiveTextColor)]
pub(crate) struct ActiveTextColor(pub(crate) Color);
