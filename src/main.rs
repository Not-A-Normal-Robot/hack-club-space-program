use bevy::prelude::*;
use hack_club_space_program::plugins::demo::DemoPlugin;

fn main() {
    App::new().add_plugins((DefaultPlugins, DemoPlugin)).run();
}
