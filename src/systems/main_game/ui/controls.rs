use crate::{
    assets::fonts::URI_FONT_WDXL_LUBRIFONT_SC,
    components::main_game::ui::controls::ControlsText,
    consts::{
        colors::{CONTROL_MODE_BACKGROUND, CONTROL_MODE_FOREGROUND},
        ui::oribar::ORIBAR_HEIGHT,
    },
    resources::{controls::GameControlMode, scene::GameScene},
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
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(8.0)),
                left: Val::ZERO,
                bottom: Val::ZERO,
                height: ORIBAR_HEIGHT * 0.5,
                border: UiRect::all(Val::Px(1.0)),
                ..Default::default()
            },
            BackgroundColor(CONTROL_MODE_BACKGROUND),
            DespawnOnExit::<GameScene>(GameScene::InGame),
            BorderColor::all(CONTROL_MODE_FOREGROUND),
            ZIndex(1),
            children![(
                Text::new(control_mode.get().to_string()),
                TextFont::from(server.load(URI_FONT_WDXL_LUBRIFONT_SC)).with_font_size(24.0),
                TextColor(CONTROL_MODE_FOREGROUND),
                ControlsText,
            )],
        ));
        return;
    };
    text.0 = control_mode.get().to_string();
}
