#![allow(dead_code)]

use core::time::Duration;

use bevy::{
    asset::{RenderAssetUsages, io::embedded::GetAssetServer},
    log::LogPlugin,
    mesh::PrimitiveTopology,
    prelude::*,
    time::TimeUpdateStrategy,
};
use hack_club_space_program::{
    components::frames::{RootSpaceLinearVelocity, RootSpacePosition},
    plugins::logic::GameLogicPlugin,
};

fn setup_time(
    mut commands: Commands,
    fixed_time: Res<Time<Fixed>>,
    mut virt_time: ResMut<Time<Virtual>>,
) {
    virt_time.advance_by(fixed_time.timestep());
    commands.insert_resource(TimeUpdateStrategy::ManualDuration(fixed_time.timestep()));
}

pub fn enable_backtrace() {
    const BACKTRACE_KEY: &str = "RUST_BACKTRACE";
    unsafe {
        if std::env::var(BACKTRACE_KEY).is_err() {
            std::env::set_var(BACKTRACE_KEY, "1");
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TestAppConfig {
    /// Whether or not the app's itme should increase
    /// every time `app.update()` is called.
    pub forward_time_on_update: bool,
    /// The log level for bevy::log
    pub log_level: Option<bevy::log::Level>,
}

impl TestAppConfig {
    pub const DEFAULT: Self = Self {
        forward_time_on_update: true,
        log_level: None,
    };
}

impl Default for TestAppConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub fn setup_default() -> App {
    setup(TestAppConfig::DEFAULT)
}

/// `forward_time_on_update`: Whether or not the app's time should
/// increase every time update() is called.
///
/// The amount of time the time is increased is by the fixed timestep
/// interval (default 64 Hz).
pub fn setup(config: TestAppConfig) -> App {
    enable_backtrace();

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameLogicPlugin));
    if let Some(level) = config.log_level {
        app.add_plugins(LogPlugin {
            level,
            ..Default::default()
        });
    }

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    if config.forward_time_on_update {
        app.add_systems(Startup, setup_time);
    }
    app.update();
    app
}

pub fn empty_mesh_material(app: &mut App) -> (Mesh2d, MeshMaterial2d<ColorMaterial>) {
    if !app.is_plugin_added::<AssetPlugin>() {
        app.add_plugins(AssetPlugin::default());
    }

    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();

    let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    let mesh = app.get_asset_server().add(mesh);
    let material = ColorMaterial::from_color(Color::WHITE);
    let material = app.get_asset_server().add(material);
    (Mesh2d(mesh), MeshMaterial2d(material))
}

pub fn assert_sv(entity: EntityRef, pos: RootSpacePosition, vel: RootSpaceLinearVelocity) {
    let name = entity
        .get::<Name>()
        .map(|name| name.into())
        .unwrap_or(entity.id().to_string());
    assert_eq!(
        entity.get::<RootSpacePosition>().copied(),
        Some(pos),
        "assertion failed: position mismatch for {name}"
    );
    assert_eq!(
        entity.get::<RootSpaceLinearVelocity>().copied(),
        Some(vel),
        "assertion failed: velocity mismatch for {name}"
    );
}

#[macro_export]
macro_rules! assert_almost_eq {
    ($x:expr, $y:expr, $tolerance:expr, $reason:expr $(, $args:expr )* $(,)?) => {
        if $x != $y {
            let diff = $x - $y;
            let rel_diff = (diff / $x.abs().max($y.abs())).abs();
            if rel_diff > $tolerance {
                panic!(
                    $reason,
                    $( $args , )*
                );
            }
        }
    };

    ($x:expr, $y:expr, $tolerance:expr $(,)?) => {
        let x = $x;
        let y = $y;
        assert_almost_eq!(
            x,
            y,
            $tolerance,
            concat!(
                "assertion failed!\n  lhs: {:?}\n  rhs: {:?}\n\n  lhs expr: ",
                stringify!($x),
                "\n  rhs expr: ",
                stringify!($y),
            ),
            x,
            y,
        )
    };
}

/// Tolerance is a fractional error that can be tolerated.
pub fn assert_sv_close(
    entity: EntityRef,
    pos: RootSpacePosition,
    vel: RootSpaceLinearVelocity,
    tolerance: f64,
) {
    let actual_pos = entity
        .get::<RootSpacePosition>()
        .copied()
        .expect("entity should have root pos");
    let actual_vel = entity
        .get::<RootSpaceLinearVelocity>()
        .copied()
        .expect("entity should have root vel");

    let dpos = actual_pos.0 - pos.0;
    let dvel = actual_vel.0 - vel.0;

    let rel_dpos = dpos.length() / ((actual_pos.0 + pos.0).length() / 2.0);
    let rel_dvel = dvel.length() / ((actual_vel.0 + vel.0).length());

    let name = entity
        .get::<Name>()
        .map(|name| name.into())
        .unwrap_or(entity.id().to_string());

    if actual_pos != pos && rel_dpos > tolerance {
        panic!(
            "position mismatch for {name}:\n
            relative position difference {rel_dpos} exceeds tolerance {tolerance}
            
            exp: {pos}
            got: {actual_pos}
            dif: {dpos}"
        );
    }

    if actual_vel != vel && rel_dvel > tolerance {
        panic!(
            "velocity mismatch for {name}:\n
            relative velocity difference {rel_dvel} exceeds tolerance {tolerance}
            
            exp: {vel}
            got: {actual_vel}
            dif: {dvel}"
        );
    }
}

/// Trait for collection of assertions.
pub trait Assertions {
    type ExtraData: Copy;
    /// Checks the app's state and panics if something's amiss.
    fn check_assertions(&self, app: &App, extra: Self::ExtraData);
}

impl<T, const N: usize> Assertions for [T; N]
where
    T: Assertions,
{
    type ExtraData = T::ExtraData;

    fn check_assertions(&self, app: &App, extra: Self::ExtraData) {
        self.iter().for_each(|assertions| {
            assertions.check_assertions(app, extra);
        });
    }
}

pub trait AssertionsCollection<'a, A>
where
    Self: IntoIterator<Item = &'a A> + Sized,
    A: Assertions + 'a,
{
    fn run_assertions_collection(self, app: &mut App, extra: A::ExtraData) {
        for (idx, assertions) in self.into_iter().enumerate() {
            eprintln!(">> Running assertions: tick {idx}");
            app.update();
            assertions.check_assertions(app, extra);
        }
    }
}

impl<'a, S, A> AssertionsCollection<'a, A> for S
where
    S: 'a,
    S: IntoIterator<Item = &'a A>,
    A: Assertions + 'a,
{
}
