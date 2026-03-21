use core::time::Duration;

// TODO: Remove this when we implement saving and multi-save-files
pub(crate) static SAVE_NAME_STR: &str = "demo";

// TODO: Remove this when we implement starting a new game
pub(crate) static DEFAULT_SAVE: &str = include_str!("./save_data.json");

/// How long to wait for save subsystem initialization to finish.
pub(crate) const INIT_TIMEOUT: Duration = Duration::from_secs(60);

#[cfg(not(target_family = "wasm"))]
pub(crate) mod nonweb {
    /// The save directory, relative to `dirs::data_dir()`.
    pub(crate) static SAVE_DIR: &str = "hack-club-space-program/saves";
}

#[cfg(target_family = "wasm")]
pub(crate) mod web {
    /// The save `IndexedDB` database name.
    pub(crate) static STORAGE_DB: &str = "io.github.not-a-normal-robot.hack-club-space-program";

    /// The current version fo the `IndexedDB` database.
    pub(crate) const STORAGE_DB_VERSION: u32 = 1;

    /// The name of the saves object store in the `IndexedDB` database.
    pub(crate) static SAVE_OBJECT_STORE: &str = "saves";

    pub(crate) static KEY_SAVE_NAME: &str = "name";
    pub(crate) static KEY_SAVE_VALUE: &str = "data";

    pub(crate) static DEFAULT_WRAPPED_SAVE: &str = concat!(
        "{\"name\":\"",
        "demo",
        "\",\"data\":",
        include_str!("./save_data.json"),
        "}",
    );
}
