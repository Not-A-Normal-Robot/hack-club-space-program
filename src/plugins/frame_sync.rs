use bevy::prelude::*;

use crate::systems::frame_sync::{
    sync_rigid_pos_to_root, sync_rigid_vel_to_root, update_active_vessel_res,
};

pub struct FrameSyncPlugin;

impl Plugin for FrameSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPreUpdate, update_active_vessel_res);
        app.add_systems(
            FixedPostUpdate,
            (sync_rigid_pos_to_root, sync_rigid_vel_to_root),
        );
    }
}
