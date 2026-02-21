use crate::plugins::physics::GamePhysicsPlugin;
use bevy::prelude::*;
use bevy_rapier2d::{prelude::*, rapier::prelude::IntegrationParameters};

/// The plugin for the game's inner logic, including
/// physics.
pub struct GameLogicPlugin;

pub const RAPIER_CONFIGURATION: RapierConfiguration = RapierConfiguration {
    gravity: Vec2::ZERO,
    physics_pipeline_active: true,
    scaled_shape_subdivision: 10,
    force_update_from_transform_changes: false,
};

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        let dt = app
            .world()
            .resource::<Time<Fixed>>()
            .timestep()
            .as_secs_f32();
        let physics = RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0)
            .in_fixed_schedule()
            .with_custom_initialization(
                RapierContextInitialization::InitializeDefaultRapierContext {
                    integration_parameters: IntegrationParameters {
                        dt,
                        max_ccd_substeps: 4,
                        num_solver_iterations: 32,
                        normalized_max_corrective_velocity: 250.0,
                        ..Default::default()
                    },
                    rapier_configuration: RAPIER_CONFIGURATION,
                },
            );

        app.add_plugins((physics, GamePhysicsPlugin));
    }
}
