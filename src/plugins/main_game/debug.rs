use bevy::prelude::*;

use crate::{
    components::main_game::{
        frames::{RigidSpaceVelocity, RootSpaceLinearVelocity, RootSpacePosition},
        relations::RailMode,
        vessel::Vessel,
    },
    resources::scene::GameScene,
};

pub(crate) struct GameDebugPlugin;

impl Plugin for GameDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, switch_game_scenes);
    }
}

fn switch_game_scenes(
    input: Res<ButtonInput<KeyCode>>,
    scene: Res<State<GameScene>>,
    mut next_scene: ResMut<NextState<GameScene>>,
) {
    if input.just_pressed(KeyCode::Delete) {
        next_scene.set(match *scene.get() {
            GameScene::MainMenu => GameScene::InGame,
            _ => GameScene::MainMenu,
        });
    }
}

fn _print_vessel_sv(
    vessels: Query<
        (
            NameOrEntity,
            &RootSpacePosition,
            &RootSpaceLinearVelocity,
            &RigidSpaceVelocity,
            &RailMode,
        ),
        With<Vessel>,
    >,
) {
    vessels.iter().for_each(|(name, pos, vel, rvel, rail)| {
        info!(
            "{name}: {pos} | {vel} | {angvel} rad/s | {rail}",
            angvel = rvel.angvel
        );
    });
}
