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
        let physics = RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0)
            .in_fixed_schedule()
            .with_custom_initialization(
                RapierContextInitialization::InitializeDefaultRapierContext {
                    integration_parameters: IntegrationParameters::default(),
                    rapier_configuration: RAPIER_CONFIGURATION,
                },
            );

        app.add_plugins((physics, GamePhysicsPlugin));
    }
}
