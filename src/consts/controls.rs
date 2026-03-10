#![cfg_attr(not(feature = "not-headless"), expect(dead_code))]

use bevy::input::keyboard::KeyCode;

/// The keys a user can press to activate a selected button.
pub(crate) const ACTIVATION_KEYCODES: [KeyCode; 3] =
    [KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space];

/// The keys a user can press to switch their mouse
/// scrolling direction.
pub(crate) const MOUSE_WHEEL_ALT_DIR: [KeyCode; 2] = [KeyCode::ControlLeft, KeyCode::ControlRight];

/// The min zoom for the simulation camera.
pub(crate) const MIN_ZOOM: f64 = 1e-20;

/// The max zoom for the simulation camera.
pub(crate) const MAX_ZOOM: f64 = 1000.0;

/// Zoom speed multiplier.
pub(crate) const ZOOM_SPEED_MULT: f64 = 12.0;

/// Camera translational movement speed multiplier.
pub(crate) const MOVE_SPEED_MULT: f64 = 36.0;

pub(crate) const SLOW_SPEED_MODIFIER: f64 = 0.025;
pub(crate) const NORMAL_SPEED_MODIFIER: f64 = 0.25;
pub(crate) const FAST_SPEED_MODIFIER: f64 = 1.0;

// Keybinds

pub(crate) const KB_MODE_SWITCH_TO_MAIN_MODE: [KeyCode; 1] = [KeyCode::Escape];
pub(crate) const KB_MODE_SWITCH_TO_MENU_MODE: [KeyCode; 1] = [KeyCode::KeyM];
pub(crate) const KB_MODE_SWITCH_TO_VESSEL_MODE: [KeyCode; 1] = [KeyCode::KeyV];
pub(crate) const KB_MODE_SWITCH_TO_CAM_MODE: [KeyCode; 1] = [KeyCode::KeyC];

pub(crate) const KB_CAM_SLOW_MOD: [KeyCode; 2] = [KeyCode::ControlLeft, KeyCode::ControlRight];
pub(crate) const KB_CAM_FAST_MOD: [KeyCode; 2] = [KeyCode::ShiftLeft, KeyCode::ShiftRight];

pub(crate) const KB_CAM_ROT_LEFT: [KeyCode; 1] = [KeyCode::KeyQ];
pub(crate) const KB_CAM_ROT_RIGHT: [KeyCode; 1] = [KeyCode::KeyE];
pub(crate) const KB_CAM_ROT_RESET: [KeyCode; 1] = [KeyCode::KeyR];

pub(crate) const KB_CAM_MOV_UP: [KeyCode; 2] = [KeyCode::KeyW, KeyCode::ArrowUp];
pub(crate) const KB_CAM_MOV_DOWN: [KeyCode; 2] = [KeyCode::KeyS, KeyCode::ArrowDown];
pub(crate) const KB_CAM_MOV_LEFT: [KeyCode; 2] = [KeyCode::KeyA, KeyCode::ArrowLeft];
pub(crate) const KB_CAM_MOV_RIGHT: [KeyCode; 2] = [KeyCode::KeyD, KeyCode::ArrowRight];
pub(crate) const KB_CAM_MOV_RESET: [KeyCode; 1] = [KeyCode::KeyC]; // "Center" camera

pub(crate) const KB_CAM_ZOOM_IN: [KeyCode; 2] = [KeyCode::Equal, KeyCode::NumpadAdd];
pub(crate) const KB_CAM_ZOOM_OUT: [KeyCode; 2] = [KeyCode::Minus, KeyCode::NumpadSubtract];
pub(crate) const KB_CAM_ZOOM_RESET: [KeyCode; 2] = [KeyCode::Digit0, KeyCode::Numpad0];

pub(crate) const KB_CAM_SWITCH_PREV: [KeyCode; 1] = [KeyCode::BracketLeft];
pub(crate) const KB_CAM_SWITCH_NEXT: [KeyCode; 1] = [KeyCode::BracketRight];

pub(crate) const KB_MENU_SWITCH_ALTIMETER_MODE: [KeyCode; 1] = [KeyCode::KeyA];
