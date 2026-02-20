use crate::components::{
    camera::{SimCamera, SimCameraOffset, SimCameraZoom},
    frames::RootSpacePosition,
};
use bevy::prelude::*;

pub mod collider;
pub mod gfx;

type CameraQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static SimCameraZoom,
        &'static mut SimCameraOffset,
        &'static Camera,
    ),
    With<SimCamera>,
>;

#[derive(Clone, Copy)]
struct GlobalData {
    zoom: SimCameraZoom,
    cam_pos: RootSpacePosition,
}
