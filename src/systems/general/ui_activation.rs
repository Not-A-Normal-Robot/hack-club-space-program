use bevy::{input::keyboard::KeyboardInput, input_focus::FocusedInput, prelude::*};

use crate::consts::controls::ACTIVATION_KEYCODES;

#[derive(EntityEvent)]
pub(crate) struct ActivationEvent {
    pub(crate) entity: Entity,
}

pub(crate) fn activation_observer_adder(
    query: Query<Entity, Added<Interaction>>,
    mut commands: Commands,
) {
    for entity in query {
        commands.entity(entity).observe(pointer_handler(entity));
        commands.entity(entity).observe(keyboard_handler(entity));
    }
}

fn pointer_handler(entity: Entity) -> impl Fn(On<Pointer<Click>>, Commands) {
    move |_: On<Pointer<Click>>, mut commands: Commands| {
        commands.trigger(ActivationEvent { entity });
    }
}

fn keyboard_handler(entity: Entity) -> impl Fn(On<FocusedInput<KeyboardInput>>, Commands) {
    move |event: On<FocusedInput<KeyboardInput>>, mut commands: Commands| {
        let input = &event.input;

        let is_activation_input = ACTIVATION_KEYCODES.contains(&input.key_code);

        if input.repeat || !input.state.is_pressed() || !is_activation_input {
            return;
        }

        commands.trigger(ActivationEvent { entity });
    }
}
