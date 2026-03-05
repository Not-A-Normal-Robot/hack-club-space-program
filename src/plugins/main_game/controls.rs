use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    components::main_game::{
        camera::Focusable, celestial::CelestialBody, relations::CelestialParent,
    },
    resources::{
        controls::{FocusableData, FocusableEntry, GameControlMode},
        scene::GameScene,
    },
    systems::main_game::{
        controls::{
            camera::control_camera, cleanup_controls, control_switching, init_controls,
            menu::control_menu,
        },
        ui::controls::update_controls_text,
    },
};

pub(crate) struct GameControlPlugin;

impl Plugin for GameControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<GameControlMode>();
        app.add_systems(OnEnter(GameScene::InGame), init_controls);
        app.add_systems(OnExit(GameScene::InGame), cleanup_controls);
        app.add_systems(
            Update,
            (
                control_switching,
                update_controls_text.run_if(state_changed::<GameControlMode>),
                control_camera.run_if(in_state(GameControlMode::CameraControl)),
                control_menu.run_if(in_state(GameControlMode::Menu)),
            )
                .run_if(in_state(GameScene::InGame)),
        );
        app.world_mut()
            .register_component_hooks::<Focusable>()
            .on_add(on_focusable_added)
            .on_remove(on_focusable_removed);
    }
}

fn on_focusable_added(mut world: DeferredWorld, ctx: HookContext) {
    let entity = ctx.entity;

    let parent = world.get::<CelestialParent>(entity).copied();
    let is_celestial_body = world.get::<CelestialBody>(entity).is_some();

    let Some(mut focusable_data) = world.get_resource_mut::<FocusableData>() else {
        return;
    };

    let new_index = if let Some(parent) = parent
        && let Some(parent_idx) = focusable_data.index_map().get(&parent.entity).copied()
    {
        (parent_idx..focusable_data.focusable_list().len())
            .find(|&i| focusable_data.get_entry(i).unwrap().is_celestial_body)
            .unwrap_or(focusable_data.focusable_list().len())
    } else {
        focusable_data.focusable_list().len()
    };

    focusable_data.insert(
        new_index,
        FocusableEntry {
            entity,
            is_celestial_body,
        },
    );
}

fn on_focusable_removed(mut world: DeferredWorld, ctx: HookContext) {
    let entity = ctx.entity;

    let Some(mut focusable_data) = world.get_resource_mut::<FocusableData>() else {
        return;
    };

    let Some(index) = focusable_data.get_index(entity) else {
        return;
    };

    focusable_data.remove(index);
}
