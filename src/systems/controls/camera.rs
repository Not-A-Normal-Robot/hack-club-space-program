use crate::{
    components::camera::{SimCamera, SimCameraOffset, SimCameraZoom},
    consts::keybinds::{
        KB_CAM_FAST_MOD, KB_CAM_ROT_LEFT, KB_CAM_ROT_RESET, KB_CAM_ROT_RIGHT, KB_CAM_SLOW_MOD,
        KB_CAM_ZOOM_IN, KB_CAM_ZOOM_OUT, KB_CAM_ZOOM_RESET,
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

const SLOW_SPEED_MODIFIER: f64 = 0.025;
const NORMAL_SPEED_MODIFIER: f64 = 0.25;
const FAST_SPEED_MODIFIER: f64 = 1.0;

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

    // Zoom: 10s/double | 1s/double | 0.25s/double
    if key.any_pressed(KB_CAM_ZOOM_OUT) {
        camera.zoom.0 = (camera.zoom.0 / 2.0f64.powf(4.0 * delta_amount)).max(1e-20);
    }
    if key.any_pressed(KB_CAM_ZOOM_IN) {
        camera.zoom.0 = (camera.zoom.0 * 2.0f64.powf(4.0 * delta_amount)).min(1e20);
    }
    if key.any_pressed(KB_CAM_ZOOM_RESET) {
        camera.zoom.0 = 1.0;
    }
}
