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

/// `forward_time_on_update`: Whether or not the app's time should
/// increase every time update() is called.
///
/// The amount of time the time is increased is by the fixed timestep
/// interval (default 64 Hz).
pub fn setup(forward_time_on_update: bool) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameLogicPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    if forward_time_on_update {
        app.add_systems(Startup, setup_time);
    }
    app.update();
    app
}
