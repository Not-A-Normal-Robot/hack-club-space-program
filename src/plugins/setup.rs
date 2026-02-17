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

// const CELESTIAL_RADIUS: f32 = 6378137.0;
// const CELESTIAL_MASS: f32 = 5.972e24;
const CELESTIAL_RADIUS: f32 = 100.0;
const CELESTIAL_MASS: f32 = 4e16;
const ALTITUDE: f32 = CELESTIAL_RADIUS + 100.0;

fn demo_startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera {
            is_active: true,
            ..Default::default()
        },
        Camera2d,
        SimCamera,
        SimCameraOffset::Detached(RootSpacePosition(DVec2::ZERO)),
        SimCameraZoom(1.0),
        Transform::from_rotation(Quat::from_rotation_z(0.0)),
    ));

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let mesh = asset_server.add(mesh);

    let material = ColorMaterial::from_color(Color::WHITE);
    let material = asset_server.add(material);

    let body = CelestialBodyBuilder {
        name: Name::new("Body"),
        radius: CELESTIAL_RADIUS,
        mass: AdditionalMassProperties::Mass(CELESTIAL_MASS),
        angle: 0.0,
        mesh: Mesh2d(mesh),
        material: MeshMaterial2d(material),
    }
    .build_with_terrain(Terrain {
        // seed: 2401,
        // octaves: 8,
        // frequency: 0.7,
        // gain: 0.5,
        // lacunarity: 0.5,
        // offset: 100.0,
        // multiplier: 100.0,
        // subdivs: 6,
        seed: 0,
        octaves: 1,
        frequency: 0.0,
        gain: 0.0,
        lacunarity: 0.0,
        offset: CELESTIAL_RADIUS as f64,
        multiplier: 0.0,
        subdivs: 2,
    });
    let body = commands.spawn(body).id();

    let vessel_pos = RootSpacePosition(DVec2::new(0.0, ALTITUDE as f64));
    let vessel_vel = RootSpaceLinearVelocity(DVec2::new(100.0, 0.0));

    let vessel = VesselBuilder {
        name: Name::new("Vessel"),
        // collider: Collider::ball(10.0),
        collider: Collider::round_cuboid(10.0, 20.0, 8.0),
        mass: AdditionalMassProperties::Mass(1e12),
        parent: CelestialParent { entity: body },
        rail_mode: RailMode::None,
        position: vessel_pos,
        linvel: vessel_vel,
        angvel: 0.0,
        angle: 0.0,
    }
    .build_rigid();
    let vessel = commands.spawn(vessel);
    let vessel_entity = vessel.id();

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
