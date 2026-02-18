use crate::{
    components::camera::{SimCamera, SimCameraOffset, SimCameraZoom},
    consts::controls::{
        FAST_SPEED_MODIFIER, KB_CAM_FAST_MOD, KB_CAM_ROT_LEFT, KB_CAM_ROT_RESET, KB_CAM_ROT_RIGHT,
        KB_CAM_SLOW_MOD, KB_CAM_ZOOM_IN, KB_CAM_ZOOM_OUT, KB_CAM_ZOOM_RESET, MAX_ZOOM, MIN_ZOOM,
        NORMAL_SPEED_MODIFIER, SLOW_SPEED_MODIFIER, ZOOM_SPEED,
    },
};
use bevy::{ecs::query::QueryData, prelude::*};
use core::f64::consts::TAU;

#[derive(QueryData)]
#[query_data(mutable)]
pub struct SimCameraInfo {
    transform: &'static mut Transform,
    offset: &'static mut SimCameraOffset,
    zoom: &'static mut SimCameraZoom,
}

type FilterSimCamera = (With<Camera>, With<SimCamera>);

#[allow(clippy::cast_possible_truncation)]
pub fn control_camera(
    mut camera: Single<SimCameraInfo, FilterSimCamera>,
    key: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
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

    // Zoom: 5s/double | 0.5s/double | 0.125s/double
    if key.any_pressed(KB_CAM_ZOOM_OUT) {
        camera.zoom.0 = (camera.zoom.0 / (ZOOM_SPEED * delta_amount).exp()).max(MIN_ZOOM);
    }
    if key.any_pressed(KB_CAM_ZOOM_IN) {
        camera.zoom.0 = (camera.zoom.0 * (ZOOM_SPEED * delta_amount).exp()).min(MAX_ZOOM);
    }
    if key.any_pressed(KB_CAM_ZOOM_RESET) {
        camera.zoom.0 = 1.0;
    }
}
