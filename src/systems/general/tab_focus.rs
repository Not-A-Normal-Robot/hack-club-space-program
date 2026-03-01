use bevy::{
    input_focus::{InputFocus, tab_navigation::TabIndex},
    prelude::*,
};

use crate::consts::TAB_FOCUS_OUTLINE;

pub(crate) fn update_tab_focus(
    mut commands: Commands,
    focus: Res<InputFocus>,
    query: Query<Entity, With<TabIndex>>,
) {
    if !focus.is_changed() {
        return;
    }

    for entity in query.iter() {
        if focus.0 == Some(entity) {
            commands.entity(entity).insert(TAB_FOCUS_OUTLINE);
        } else {
            commands.entity(entity).try_remove::<Outline>();
        }
    }
}
