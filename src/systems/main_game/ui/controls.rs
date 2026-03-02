use crate::{
    assets::fonts::URI_FONT_WDXL_LUBRIFONT_SC, components::main_game::ui::controls::ControlsText,
    resources::controls::GameControlMode,
};
use bevy::prelude::*;

pub(crate) fn update_controls_text(
    mut commands: Commands,
    mut text: Query<&mut Text, With<ControlsText>>,
    control_mode: Res<State<GameControlMode>>,
    server: Res<AssetServer>,
) {
    let Some(mut text) = text.iter_mut().next() else {
        commands.spawn((
            Node::DEFAULT,
            Text::new(control_mode.get().to_string()),
            TextFont::from(server.load(URI_FONT_WDXL_LUBRIFONT_SC)),
            ControlsText,
        ));
        return;
    };
    text.0 = control_mode.get().to_string();
}
