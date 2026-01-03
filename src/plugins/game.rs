use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

fn demo_startup(mut commands: Commands) {
    commands.spawn((
        Camera::default(),
        Camera2d,
        Transform::from_rotation(Quat::from_rotation_z(0.0)),
    ));

    commands.spawn((
        Collider::cuboid(500.0, 10.0),
        Transform::from_xyz(0.0, -100.0, 0.0),
    ));
    commands.spawn((
        RigidBody::Dynamic,
        Velocity::zero(),
        Collider::ball(10.0),
        Restitution::coefficient(0.7),
        Transform::from_xyz(0.0, 900.0, 0.0),
        AdditionalMassProperties::Mass(2.0),
    ));
}

fn demo_print(positions: Query<&Transform, With<RigidBody>>) {
    for transform in positions.iter() {
        println!("Ball altitude: {}", transform.translation.y);
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1000.0))
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(Startup, demo_startup)
            .add_systems(FixedUpdate, demo_print);
    }
}
