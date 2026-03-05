use bevy::prelude::*;

use crate::{consts::controls::KB_MENU_SWITCH_ALTIMETER_MODE, resources::ui::AltimeterMode};

pub(crate) fn control_menu(
    key: Res<ButtonInput<KeyCode>>,
    altimeter_mode: Res<State<AltimeterMode>>,
    mut next_altimeter_mode: ResMut<NextState<AltimeterMode>>,
) {
    if key.any_just_pressed(KB_MENU_SWITCH_ALTIMETER_MODE) {
        next_altimeter_mode.set(altimeter_mode.next());
    }
}
