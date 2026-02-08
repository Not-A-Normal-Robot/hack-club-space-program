use crate::resources::GameControlMode;
use bevy::prelude::*;

#[derive(Component)]
pub struct ControlsText;

pub fn update_controls_text(
    mut commands: Commands,
    mut text: Query<&mut Text, With<ControlsText>>,
    control_mode: Res<State<GameControlMode>>,
) {
    let Some(mut text) = text.iter_mut().next() else {
        commands.spawn((
            Node::DEFAULT,
            Text::new(control_mode.get().to_str()),
            ControlsText,
        ));
        return;
    };
    text.0.clear();
    text.0.push_str(control_mode.to_str());
}
