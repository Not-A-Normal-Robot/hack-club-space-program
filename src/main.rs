use bevy::prelude::*;
use hack_club_space_program::plugins::setup::GameSetupPlugin;

fn enable_backtrace() {
    const BACKTRACE_KEY: &str = "RUST_BACKTRACE";
    unsafe {
        if std::env::var(BACKTRACE_KEY).is_err() {
            std::env::set_var(BACKTRACE_KEY, "1");
        }
    }
}

fn main() {
    enable_backtrace();

    App::new()
        .add_plugins((DefaultPlugins, GameSetupPlugin))
        .run();
}
