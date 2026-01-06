use bevy::prelude::*;
use bevy_rapier2d::plugin::NoUserData;

use crate::systems::frame_sync::{
    apply_root_velocity, post_rapier_frame_switch, pre_rapier_frame_switch, sync_rigid_pos_to_root,
    sync_rigid_vel_to_root, update_active_vessel_res,
};

pub struct FrameSyncPlugin;

impl Plugin for FrameSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPreUpdate, update_active_vessel_res);
        app.add_systems(
            FixedUpdate,
            pre_rapier_frame_switch
                .before(bevy_rapier2d::prelude::systems::step_simulation::<NoUserData>),
        );
        app.add_systems(
            FixedUpdate,
            post_rapier_frame_switch
                .after(bevy_rapier2d::prelude::systems::step_simulation::<NoUserData>),
        );
        app.add_systems(
            FixedPostUpdate,
            (
                (sync_rigid_pos_to_root, sync_rigid_vel_to_root),
                apply_root_velocity,
            )
                .chain(),
        );
    }
}
