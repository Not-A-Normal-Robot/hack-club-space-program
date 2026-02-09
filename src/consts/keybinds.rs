use bevy::input::keyboard::KeyCode;

pub const KB_MODE_SWITCH_TO_MAIN_MODE: [KeyCode; 1] = [KeyCode::Escape];
pub const KB_MODE_SWITCH_TO_MENU_MODE: [KeyCode; 1] = [KeyCode::KeyM];
pub const KB_MODE_SWITCH_TO_VESSEL_MODE: [KeyCode; 1] = [KeyCode::KeyV];
pub const KB_MODE_SWITCH_TO_CAM_MODE: [KeyCode; 1] = [KeyCode::KeyC];

pub const KB_CAM_SLOW_MOD: [KeyCode; 2] = [KeyCode::ControlLeft, KeyCode::ControlRight];
pub const KB_CAM_FAST_MOD: [KeyCode; 2] = [KeyCode::ShiftLeft, KeyCode::ShiftRight];

pub const KB_CAM_ROT_LEFT: [KeyCode; 1] = [KeyCode::KeyQ];
pub const KB_CAM_ROT_RIGHT: [KeyCode; 1] = [KeyCode::KeyE];
pub const KB_CAM_ROT_RESET: [KeyCode; 1] = [KeyCode::KeyR];

pub const KB_CAM_MOV_UP: [KeyCode; 2] = [KeyCode::KeyW, KeyCode::ArrowUp];
pub const KB_CAM_MOV_DOWN: [KeyCode; 2] = [KeyCode::KeyS, KeyCode::ArrowDown];
pub const KB_CAM_MOV_LEFT: [KeyCode; 2] = [KeyCode::KeyA, KeyCode::ArrowLeft];
pub const KB_CAM_MOV_RIGHT: [KeyCode; 2] = [KeyCode::KeyD, KeyCode::ArrowRight];

pub const KB_CAM_ZOOM_IN: [KeyCode; 2] = [KeyCode::Equal, KeyCode::NumpadAdd];
pub const KB_CAM_ZOOM_OUT: [KeyCode; 2] = [KeyCode::Minus, KeyCode::NumpadSubtract];
pub const KB_CAM_ZOOM_RESET: [KeyCode; 2] = [KeyCode::Digit0, KeyCode::Numpad0];
