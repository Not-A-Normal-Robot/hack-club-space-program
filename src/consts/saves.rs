// TODO: Remove this when we implement saving and multi-save-files
pub(crate) static SAVE_NAME_STR: &str = "demo";

// TODO: Remove this when we implement starting a new game
pub(crate) static DEFAULT_SAVE: &str = include_str!("./save_data.json");

/// The save directory, relative to `dirs::data_dir()`.
#[cfg(not(target_family = "wasm"))]
pub(crate) static SAVE_DIR: &str = "hack-club-space-program/saves";
