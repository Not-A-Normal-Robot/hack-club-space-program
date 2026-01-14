use bevy::prelude::*;
use bevy_rapier2d::plugin::NoUserData;

use crate::systems::frame_sync::{
    apply_root_velocity, post_rapier_frame_switch, pre_rapier_frame_switch, sync_rigid_pos_to_root,
    sync_rigid_vel_to_root, sync_root_to_rigid, update_active_vessel_resource,
};

pub struct FrameSyncPlugin;

impl Plugin for FrameSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (
                update_active_vessel_resource,
                sync_root_to_rigid,
                pre_rapier_frame_switch,
            )
                .chain(),
        );
        // app.add_systems(
        //     FixedUpdate,
        //     (sync_root_to_rigid, pre_rapier_frame_switch)
        //         .chain()
        //         .before(bevy_rapier2d::prelude::systems::step_simulation::<NoUserData>),
        // );
        app.add_systems(
            FixedPostUpdate,
            (
                sync_rigid_vel_to_root,
                sync_rigid_pos_to_root,
                post_rapier_frame_switch,
                apply_root_velocity,
            )
                .chain()
                .after(bevy_rapier2d::prelude::systems::step_simulation::<NoUserData>),
        );
    }
}
