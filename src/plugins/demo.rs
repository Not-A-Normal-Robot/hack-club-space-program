use bevy::prelude::*;

const CIRCLE_RADIUS: f32 = 1e3;

fn add_demo_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera {
            is_active: true,
            ..Default::default()
        },
        Camera2d,
    ));
    let rect_mesh = meshes.add(Rectangle::new(4.0, 8.0));
    let circle_mesh = meshes.add(Circle::new(CIRCLE_RADIUS));
    let material = materials.add(Color::WHITE);

    commands.spawn((Mesh2d(rect_mesh), MeshMaterial2d(material.clone())));
    commands.spawn((
        Mesh2d(circle_mesh),
        MeshMaterial2d(material),
        Transform::from_xyz(CIRCLE_RADIUS + 10.0, 0.0, 0.0),
    ));
}

pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_demo_objects);
    }
}
