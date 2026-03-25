use core::time::Duration;

// TODO: Remove this when we implement saving and multi-save-files
#[allow(dead_code)]
pub(crate) static SAVE_NAME_STR: &str = "demo";

// TODO: Remove this when we implement starting a new game
/// The default save data, cbor-encoded, zlib-compressed.
#[allow(dead_code)]
pub(crate) static DEFAULT_SAVE_ZLIB_CBOR: &[u8] =
    include_bytes!("../../assets/_processed/default_save.cbor.zz");

/// How long to wait for save subsystem initialization to finish.
#[expect(dead_code)]
pub(crate) const INIT_TIMEOUT: Duration = Duration::from_secs(60);

#[cfg(not(target_family = "wasm"))]
pub(crate) mod nonweb {
    /// The save directory, relative to `dirs::data_dir()`.
    pub(crate) static SAVE_DIR: &str = "hack-club-space-program/saves";
}

#[cfg(target_family = "wasm")]
pub(crate) mod web {
    use std::borrow::Cow;

    use super::*;
    use crate::storage::{SaveDataKind, web::WrappedData};
    use wasm_bindgen::JsValue;

    /// The save `IndexedDB` database name.
    pub(crate) static STORAGE_DB: &str = if cfg!(test) {
        "io.github.not-a-normal-robot.hack-club-space-program.testing-db"
    } else {
        "io.github.not-a-normal-robot.hack-club-space-program"
    };

    /// The current version fo the `IndexedDB` database.
    pub(crate) const STORAGE_DB_VERSION: u32 = 1;

    /// The name of the saves object store in the `IndexedDB` database.
    pub(crate) static SAVE_OBJECT_STORE_NAME: &str = "saves";

    pub(crate) static KEY_SAVE_NAME: &str = "name";
    pub(crate) static KEY_SAVE_VALUE: &str = "data";

    pub(crate) fn get_default_wrapped_save() -> JsValue {
        serde_wasm_bindgen::to_value(&WrappedData {
            save_name: String::from(SAVE_NAME_STR),
            save_data_kind: SaveDataKind::MainSave,
            data: Cow::Borrowed(DEFAULT_SAVE_ZLIB_CBOR),
        })
        .expect("wrapped save data should be serializable")
    }
}
