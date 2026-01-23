use bevy::prelude::*;

use crate::components::{
    frames::{RootSpaceLinearVelocity, RootSpacePosition},
    relations::RailMode,
    vessel::Vessel,
};

pub struct HcspDebugPlugin;

impl Plugin for HcspDebugPlugin {
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
            &RailMode,
        ),
        With<Vessel>,
    >,
) {
    vessels.iter().for_each(|(name, pos, vel, rail)| {
        info!("{name}: {pos} | {vel} | {rail}");
    });
}
