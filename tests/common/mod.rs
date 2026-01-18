#![allow(dead_code)]

use core::time::Duration;

use bevy::{prelude::*, time::TimeUpdateStrategy};
use hack_club_space_program::plugins::game::GameLogicPlugin;

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

/// `forward_time_on_update`: Whether or not the app's time should
/// increase every time update() is called.
///
/// The amount of time the time is increased is by the fixed timestep
/// interval (default 64 Hz).
pub fn setup(forward_time_on_update: bool) -> App {
    enable_backtrace();

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameLogicPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    if forward_time_on_update {
        app.add_systems(Startup, setup_time);
    }
    app.update();
    app
}

/// Trait for collection of assertions.
pub trait Assertions {
    type ExtraData: Copy;
    /// Checks the app's state and panics if something's amiss.
    fn check_assertions(&self, app: &App, extra: Self::ExtraData);
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
