use bevy::prelude::*;

use crate::{
    resources::scene::GameScene,
    systems::main_game::{
        frame_sync::{
            post_rapier_frame_switch, pre_rapier_frame_switch, update_active_vessel_resource,
            write_rigid_pos_to_root, write_rigid_vel_to_root,
        },
        gravity::apply_gravity_and_velocity,
        rail::{write_rail_to_sv, write_sv_to_rail},
        terrain::collider::update_terrain_colliders,
    },
};

pub(crate) struct GamePhysicsPlugin;

impl Plugin for GamePhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (
                write_rail_to_sv,
                apply_gravity_and_velocity,
                update_active_vessel_resource,
                (pre_rapier_frame_switch, update_terrain_colliders),
            )
                .chain()
                .run_if(in_state(GameScene::InGame)),
        );
        app.add_systems(
            FixedPostUpdate,
            (
                (write_rigid_vel_to_root, write_rigid_pos_to_root),
                (post_rapier_frame_switch, write_sv_to_rail),
            )
                .chain()
                .run_if(in_state(GameScene::InGame)),
        );
    }
}
