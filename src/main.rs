use bevy::prelude::*;
use hack_club_space_program::plugins::game::GamePlugin;

fn main() {
    App::new().add_plugins((DefaultPlugins, GamePlugin)).run();
}
