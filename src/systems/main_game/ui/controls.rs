use crate::{
    assets::fonts::URI_FONT_WDXL_LUBRIFONT_SC,
    resources::{controls::GameControlMode, scene::GameScene},
};
use bevy::prelude::*;

#[derive(Component)]
#[require(DespawnOnExit::<GameScene>(GameScene::InGame))]
pub struct ControlsText;

pub fn update_controls_text(
    mut commands: Commands,
    mut text: Query<&mut Text, With<ControlsText>>,
    control_mode: Res<State<GameControlMode>>,
    server: Res<AssetServer>,
) {
    let Some(mut text) = text.iter_mut().next() else {
        commands.spawn((
            Node::DEFAULT,
            Text::new(control_mode.get().to_str()),
            TextFont::from(server.load(URI_FONT_WDXL_LUBRIFONT_SC)),
            ControlsText,
        ));
        return;
    };
    text.0.clear();
    text.0.push_str(control_mode.to_str());
}
