use bevy::prelude::*;

use crate::components::{
    frames::{RigidSpaceVelocity, RootSpaceLinearVelocity, RootSpacePosition},
    relations::RailMode,
    vessel::Vessel,
};

pub struct GameDebugPlugin;

impl Plugin for GameDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, print_vessel_sv);
    }
}

fn print_vessel_sv(
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
