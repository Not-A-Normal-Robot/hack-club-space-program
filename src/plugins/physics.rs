use bevy::prelude::*;

use crate::systems::{
    frame_sync::{
        apply_root_velocity, post_rapier_frame_switch, pre_rapier_frame_switch,
        update_active_vessel_resource, write_rigid_pos_to_root, write_rigid_vel_to_root,
    },
    gravity::{apply_gravity, unapply_gravity_to_unloaded},
    rail::{write_rail_to_sv, write_sv_to_rail},
};

pub struct HcspPhysicsPlugin;

impl Plugin for HcspPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (
                (apply_gravity, unapply_gravity_to_unloaded, write_rail_to_sv),
                apply_root_velocity,
                update_active_vessel_resource,
                pre_rapier_frame_switch,
            )
                .chain(),
        );
        app.add_systems(
            FixedPostUpdate,
            (
                (write_rigid_vel_to_root, write_rigid_pos_to_root),
                (post_rapier_frame_switch, write_sv_to_rail),
            )
                .chain(),
        );
    }
}
