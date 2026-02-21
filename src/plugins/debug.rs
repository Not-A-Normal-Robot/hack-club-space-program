use bevy::prelude::*;
use bevy_rapier2d::render::{
    ColliderDebug, DebugRenderMode, DebugRenderStyle, RapierDebugRenderPlugin,
};

use crate::components::{
    frames::{RigidSpaceVelocity, RootSpaceLinearVelocity, RootSpacePosition},
    relations::RailMode,
    vessel::Vessel,
};

pub struct GameDebugPlugin;

impl Plugin for GameDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierDebugRenderPlugin {
            enabled: true,
            default_collider_debug: ColliderDebug::AlwaysRender,
            mode: DebugRenderMode::all(),
            style: DebugRenderStyle {
                rigid_body_axes_length: 20.0,
                subdivisions: 512,
                border_subdivisions: 20,
                collider_aabb_color: [0.0, 0.0, 0.0, 0.0],
                ..Default::default()
            },
        });
        // app.add_systems(FixedPostUpdate, _print_vessel_sv);
    }
}

fn _print_vessel_sv(
    vessels: Query<
        (
            NameOrEntity,
            &RootSpacePosition,
            &RootSpaceLinearVelocity,
            &RigidSpaceVelocity,
            &RailMode,
        ),
        With<Vessel>,
    >,
) {
    vessels.iter().for_each(|(name, pos, vel, rvel, rail)| {
        info!(
            "{name}: {pos} | {vel} | {angvel} rad/s | {rail}",
            angvel = rvel.angvel
        );
    });
}
