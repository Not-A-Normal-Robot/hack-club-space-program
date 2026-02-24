use crate::{
    components::{
        camera::{Focusable, SimCamera, SimCameraOffset, SimCameraZoom},
        celestial::CelestialBody,
        frames::RootSpacePosition,
        relations::CelestialParent,
        vessel::Vessel,
    },
    consts::controls::{
        FAST_SPEED_MODIFIER, KB_CAM_FAST_MOD, KB_CAM_MOV_DOWN, KB_CAM_MOV_LEFT, KB_CAM_MOV_RESET,
        KB_CAM_MOV_RIGHT, KB_CAM_MOV_UP, KB_CAM_ROT_LEFT, KB_CAM_ROT_RESET, KB_CAM_ROT_RIGHT,
        KB_CAM_SLOW_MOD, KB_CAM_SWITCH_NEXT, KB_CAM_SWITCH_PREV, KB_CAM_ZOOM_IN, KB_CAM_ZOOM_OUT,
        KB_CAM_ZOOM_RESET, MAX_ZOOM, MIN_ZOOM, MOVE_SPEED_MULT, NORMAL_SPEED_MODIFIER,
        SLOW_SPEED_MODIFIER, ZOOM_SPEED_MULT,
    },
    math::quat_to_rot,
    resources::controls::{FocusableData, FocusableEntry},
};
use bevy::{ecs::query::QueryData, math::DVec2, prelude::*};
use core::{cmp::Ordering, f64::consts::TAU};

#[derive(QueryData)]
#[query_data(mutable)]
pub struct SimCameraInfo {
    transform: &'static mut Transform,
    offset: &'static mut SimCameraOffset,
    zoom: &'static mut SimCameraZoom,
}

type FilterSimCamera = (
    With<Camera>,
    With<SimCamera>,
    Without<Vessel>,
    Without<CelestialBody>,
    Without<Focusable>,
);

type FocusableQuery<'w, 's> = Query<'w, 's, (Entity, &'static RootSpacePosition), With<Focusable>>;

#[derive(Clone, Copy)]
enum SwitchDirection {
    Prev,
    Next,
}

fn focus_closest(
    mut offset: Mut<SimCameraOffset>,
    current_pos: RootSpacePosition,
    focusables: FocusableQuery,
) {
    let closest = focusables.into_iter().min_by(|(_, pos_a), (_, pos_b)| {
        pos_a
            .distance_squared(current_pos.0)
            .partial_cmp(&pos_b.distance_squared(current_pos.0))
            .unwrap_or(Ordering::Equal)
    });

    if let Some((entity, position)) = closest {
        *offset = SimCameraOffset::Attached {
            entity,
            last_known_pos: *position,
            offset: DVec2::ZERO,
        };
    }
}

fn switch_focus(
    mut offset: Mut<SimCameraOffset>,
    current_pos: RootSpacePosition,
    focusables: FocusableQuery,
    direction: SwitchDirection,
    focusable_data: &FocusableData,
) {
    let current_attachment = match *offset {
        SimCameraOffset::Attached { entity, .. } => entity,
        SimCameraOffset::Detached(..) => {
            focus_closest(offset, current_pos, focusables);
            return;
        }
    };

    let Some(index) = focusable_data.index_map.get(&current_attachment).copied() else {
        error!("Camera is attached to an unindexed entity; switching to closest focusable entity");
        focus_closest(offset, current_pos, focusables);
        return;
    };

    if focusable_data.list.is_empty() {
        info!("Attempted to switch focus when there is nothing valid to switch to");
        return;
    }

    let new_index = match direction {
        SwitchDirection::Next => (index + 1) % focusable_data.list.len(),
        SwitchDirection::Prev => index
            .checked_sub(1)
            .unwrap_or(focusable_data.list.len() - 1),
    };

    let new_entity = focusable_data.list[new_index].entity;
    let new_position = focusables.get(new_entity).map_or_else(
        |_| {
            error!(
                "Entity {new_entity} is in the focusable data resource yet wasn't found in the query"
            );
            RootSpacePosition(DVec2::ZERO)
        },
        |(_, &p)| p,
    );

    *offset = SimCameraOffset::Attached {
        entity: new_entity,
        last_known_pos: new_position,
        offset: DVec2::ZERO,
    }
}

#[expect(clippy::cast_possible_truncation)]
pub fn control_camera(
    mut camera: Single<SimCameraInfo, FilterSimCamera>,
    key: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut queries: ParamSet<(Query<&RootSpacePosition>, FocusableQuery)>,
    focusable_data: Res<FocusableData>,
) {
    let speed_mult = if key.any_pressed(KB_CAM_SLOW_MOD) {
        SLOW_SPEED_MODIFIER
    } else if key.any_pressed(KB_CAM_FAST_MOD) {
        FAST_SPEED_MODIFIER
    } else {
        NORMAL_SPEED_MODIFIER
    };

    let delta_amount = time.delta_secs_f64() * speed_mult;

    // Camera: 40s/rev | 4s/rev | 1s/rev
    if key.any_pressed(KB_CAM_ROT_LEFT) {
        camera.transform.rotate_z((delta_amount * TAU) as f32);
    }
    if key.any_pressed(KB_CAM_ROT_RIGHT) {
        camera.transform.rotate_z((-delta_amount * TAU) as f32);
    }
    if key.any_pressed(KB_CAM_ROT_RESET) {
        camera.transform.rotation = Quat::IDENTITY;
    }

    let cam_rotation = quat_to_rot(camera.transform.rotation);

    // Zoom: 5s/double | 0.5s/double | 0.125s/double
    if key.any_pressed(KB_CAM_ZOOM_OUT) {
        camera.zoom.0 = (camera.zoom.0 / (ZOOM_SPEED_MULT * delta_amount).exp()).max(MIN_ZOOM);
    }
    if key.any_pressed(KB_CAM_ZOOM_IN) {
        camera.zoom.0 = (camera.zoom.0 * (ZOOM_SPEED_MULT * delta_amount).exp()).min(MAX_ZOOM);
    }
    if key.any_pressed(KB_CAM_ZOOM_RESET) {
        camera.zoom.0 = 1.0;
    }

    // Movement
    let movement_speed = MOVE_SPEED_MULT * speed_mult / camera.zoom.0;
    let mut movement_delta = DVec2::ZERO;

    if key.any_pressed(KB_CAM_MOV_UP) {
        movement_delta += DVec2::new(0.0, movement_speed);
    }
    if key.any_pressed(KB_CAM_MOV_DOWN) {
        movement_delta += DVec2::new(0.0, -movement_speed);
    }
    if key.any_pressed(KB_CAM_MOV_LEFT) {
        movement_delta += DVec2::new(-movement_speed, 0.0);
    }
    if key.any_pressed(KB_CAM_MOV_RIGHT) {
        movement_delta += DVec2::new(movement_speed, 0.0);
    }

    let movement_delta = DVec2::from_angle(cam_rotation).rotate(movement_delta);

    match &mut *camera.offset {
        SimCameraOffset::Attached { offset, .. } => *offset += movement_delta,
        SimCameraOffset::Detached(pos) => pos.0 += movement_delta,
    }

    if key.any_just_pressed(KB_CAM_MOV_RESET) {
        match &mut *camera.offset {
            SimCameraOffset::Attached { offset, .. } => *offset = DVec2::ZERO,
            SimCameraOffset::Detached(pos) => pos.0 = DVec2::ZERO,
        }
    }

    // Focus switching
    let current_pos = camera.offset.mutably().get_root_position(queries.p0());

    if key.any_just_pressed(KB_CAM_SWITCH_PREV) {
        switch_focus(
            camera.offset.reborrow(),
            current_pos,
            queries.p1(),
            SwitchDirection::Prev,
            &focusable_data,
        );
    }
    if key.any_just_pressed(KB_CAM_SWITCH_NEXT) {
        switch_focus(
            camera.offset.reborrow(),
            current_pos,
            queries.p1(),
            SwitchDirection::Next,
            &focusable_data,
        );
    }
}

type FocusableDataQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Option<&'static CelestialParent>,
        Option<&'static CelestialBody>,
    ),
    Added<Focusable>,
>;

pub fn update_focusable_data(
    added: FocusableDataQuery,
    mut removed: RemovedComponents<Focusable>,
    mut resource: ResMut<FocusableData>,
) {
    for r in removed.read() {
        let Some(index) = resource.index_map.get(&r).copied() else {
            continue;
        };

        resource.index_map.remove(&r);
        resource.list.remove(index);
    }

    for (entity, parent, body) in added {
        let Some(parent_idx) = parent
            .and_then(|p| resource.index_map.get(&p.entity))
            .copied()
        else {
            let index = resource.list.len();
            resource.list.push(FocusableEntry {
                entity,
                is_celestial_body: body.is_some(),
            });
            resource.index_map.insert(entity, index);
            continue;
        };

        let mut new_index = resource.list.len();

        for i in parent_idx..resource.list.len() {
            if resource.list[i].is_celestial_body {
                new_index = i;
                break;
            }
        }

        resource.list.insert(
            new_index,
            FocusableEntry {
                entity,
                is_celestial_body: body.is_some(),
            },
        );
    }
}
