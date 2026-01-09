use bevy::prelude::*;
use hack_club_space_program::plugins::game::{GameLogicPlugin, GameSetupPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, GameLogicPlugin, GameSetupPlugin))
        .run();
}
