use crate::{
    builders::{celestial::CelestialBodyBuilder, vessel::VesselBuilder},
    components::{
        camera::{SimCamera, SimCameraOffset, SimCameraZoom},
        celestial::Terrain,
        frames::{RootSpaceLinearVelocity, RootSpacePosition},
        relations::{CelestialParent, RailMode},
    },
    plugins::{
        controls::GameControlPlugin, debug::GameDebugPlugin, logic::GameLogicPlugin,
        render::GameRenderPlugin,
    },
    resources::ActiveVessel,
};
#[cfg(feature = "trace")]
use bevy::log::Level;
use bevy::{asset::RenderAssetUsages, math::DVec2, mesh::PrimitiveTopology};
use bevy::{log::LogPlugin, prelude::*};
use bevy_rapier2d::prelude::*;

const CELESTIAL_RADIUS: f32 = 6378137.0;
const CELESTIAL_MASS: f32 = 5.972e24;
const ALTITUDE: f32 = CELESTIAL_RADIUS + 100.0;

fn demo_startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let mesh = meshes.add(mesh);

    let material = ColorMaterial::from_color(Color::srgba(1.0, 1.0, 1.0, 0.2));
    let material = materials.add(material);

    let body = CelestialBodyBuilder {
        name: Name::new("Body"),
        radius: CELESTIAL_RADIUS,
        mass: AdditionalMassProperties::Mass(CELESTIAL_MASS),
        angle: 0.0,
        mesh: Mesh2d(mesh),
        material: MeshMaterial2d(material.clone()),
    }
    .build_with_terrain(Terrain {
        seed: 2401,
        octaves: 6,
        frequency: 400.0,
        gain: 0.4,
        lacunarity: 0.6,
        offset: CELESTIAL_RADIUS as f64,
        multiplier: CELESTIAL_RADIUS as f64 * 0.001,
        subdivs: 6,
    });
    let body = commands.spawn(body).id();

    let vessel_pos = RootSpacePosition(DVec2::new(0.0, ALTITUDE as f64));
    let vessel_vel = RootSpaceLinearVelocity(DVec2::new(100.0, 0.0));
    let vessel_half_x = 10.0;
    let vessel_half_y = 20.0;

    let mesh = Mesh2d(meshes.add(Rectangle::new(vessel_half_x * 2.0, vessel_half_y * 2.0)));

    let vessel = VesselBuilder {
        name: Name::new("Vessel"),
        collider: Collider::cuboid(vessel_half_x, vessel_half_y),
        mass: AdditionalMassProperties::Mass(1e12),
        parent: CelestialParent { entity: body },
        rail_mode: RailMode::None,
        position: vessel_pos,
        linvel: vessel_vel,
        angvel: 129.0,
        angle: 0.0,
        mesh,
        material: MeshMaterial2d(material),
    }
    .build_rigid();
    let vessel = commands.spawn(vessel);
    let vessel_entity = vessel.id();

    commands.spawn((
        Camera {
            is_active: true,
            ..Default::default()
        },
        Camera2d,
        SimCamera,
        SimCameraOffset::Attached {
            entity: vessel_entity,
            last_known_pos: vessel_pos,
            offset: DVec2::ZERO,
        },
        SimCameraZoom(1.0),
        Transform::IDENTITY,
    ));

    commands.insert_resource(ActiveVessel {
        entity: vessel_entity,
        prev_tick_parent: body,
        prev_tick_position: vessel_pos,
        prev_tick_velocity: vessel_vel,
    });
}

/// The entry point for the full game as a plugin.
///
/// Automatically initializes all other plugins for the game.
pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(LogPlugin {
            #[cfg(feature = "trace")]
            level: Level::TRACE,
            ..Default::default()
        }));
        app.add_systems(Startup, demo_startup);
        app.add_plugins(RapierDebugRenderPlugin {
            enabled: true,
            default_collider_debug: ColliderDebug::AlwaysRender,
            mode: DebugRenderMode::all(),
            style: DebugRenderStyle {
                rigid_body_axes_length: 20.0,
                subdivisions: 512,
                border_subdivisions: 20,
                collider_aabb_color: [0.0, 0.0, 0.0, 0.0],
                ..Default::default()
            },
        });
        app.add_plugins((
            GameLogicPlugin,
            GameDebugPlugin,
            GameControlPlugin,
            GameRenderPlugin,
        ));
    }
}
