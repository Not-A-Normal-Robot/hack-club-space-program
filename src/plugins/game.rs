use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::{CelestialBody, Heightmap, ParentBody, Vessel},
    plugins::frame_sync::FrameSyncPlugin,
};

const DEMO_HEIGHTMAP: [f64; 10] = [10.0, 10.0, 10.0, 10.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0];

fn demo_startup(mut commands: Commands) {
    commands.spawn((
        Camera::default(),
        Camera2d,
        Transform::from_rotation(Quat::from_rotation_z(0.0)),
    ));
    let body = commands
        .spawn((
            CelestialBody { radius: 10.0 },
            AdditionalMassProperties::Mass(10.0),
            Heightmap(Box::from(DEMO_HEIGHTMAP)),
        ))
        .id();
    commands.spawn((
        Vessel,
        Collider::ball(10.0),
        AdditionalMassProperties::Mass(1e4),
        ParentBody(body),
    ));
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1000.0))
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_plugins(FrameSyncPlugin)
            .add_systems(Startup, demo_startup);
    }
}
