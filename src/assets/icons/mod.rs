use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use core::f32::consts::PI;
use std::sync::LazyLock;

use crate::consts::colors::icons::COLOR_ICON_PROGRADE;

pub(crate) static ICON_PROGRADE: LazyLock<Shape> = LazyLock::new(|| {
    let path = ShapePath::new()
        .move_to(Vec2::new(-30.0, 0.0))
        .line_to(Vec2::new(-12.0, 0.0))
        .move_to(Vec2::new(12.0, 0.0))
        .line_to(Vec2::new(30.0, 0.0))
        .move_to(Vec2::new(0.0, -30.0))
        .line_to(Vec2::new(0.0, -12.0));

    // let circle = Circle::new(12.0);
    let circle = ShapePath::new()
        .move_to(Vec2::new(-12.0, 0.0))
        .arc(Vec2::ZERO, Vec2::splat(12.0), PI, 0.0)
        .arc(Vec2::ZERO, Vec2::splat(12.0), PI, 0.0);

    ShapeBuilder::with(&path)
        .add(&circle)
        .fill(Color::NONE)
        .stroke(Stroke::new(COLOR_ICON_PROGRADE, 5.0))
        .build()
});

pub(super) fn initialize_icons() {
    LazyLock::force(&ICON_PROGRADE);
}
