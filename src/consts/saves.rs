use core::time::Duration;

// TODO: Remove this when we implement saving and multi-save-files
#[allow(dead_code)]
pub(crate) static SAVE_NAME_STR: &str = "demo";

// TODO: Remove this when we implement starting a new game
/// The default save data, cbor-encoded, zlib-compressed.
#[allow(dead_code)]
pub(crate) static DEFAULT_SAVE_ZLIB_CBOR: &[u8] =
    include_bytes!("../../assets/_processed/default_save.cbor.zlib");

/// How long to wait for save subsystem initialization to finish.
#[expect(dead_code)]
pub(crate) const INIT_TIMEOUT: Duration = Duration::from_secs(60);

#[cfg(not(target_family = "wasm"))]
pub(crate) mod nonweb {
    /// The storage directory, relative to `dirs::data_dir()`.
    pub(crate) static STORAGE_DIR: &str = "hack-club-space-program";

    /// The saves directory, relative to the storage directory.
    ///
    /// Contains many directories, each for a specific save.
    pub(crate) static SAVES_DIR: &str = "saves";

    /// The main save file's name, relative to the specific save's directory.
    pub(crate) static MAIN_SAVE_FILE_NAME: &str = "main.cbor.zlib";
}

#[cfg(target_family = "wasm")]
pub(crate) mod web {
    use std::borrow::Cow;

    use super::*;
    use crate::storage::{
        SaveDataKind,
        web::{SaveDataDiscrim, WrappedData},
    };
    use serde_bytes::Bytes;
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

    /// The object key for the name of the save.
    pub(crate) static KEY_SAVE_NAME: &str = "name";
    /// The object key for the kind of object this is.
    pub(crate) static KEY_SAVE_KIND: &str = "kind";
    /// The object key for the discriminator.
    pub(crate) static KEY_SAVE_DISCRIM: &str = "discrim";
    /// The object key that indexes into a `Uint8Array` of
    /// zlib-compressed CBOR-encoded save data.
    pub(crate) static KEY_SAVE_VALUE: &str = "data";

    /// Index based only on name.
    ///
    /// Used for e.g. getting the list of objects
    /// associated with the save.
    pub(crate) static INDEX_NAME_ONLY: &str = "name";
    /// Index based only on names and kinds.
    ///
    /// Used for e.g. getting the list of quicksaves.
    pub(crate) static INDEX_NAME_KIND: &str = "name,kind";
    /// Index based on kind, discrim, and name, in that order.
    ///
    /// Used for e.g. listing the names of saves.
    pub(crate) static INDEX_KIND_DISCRIM_NAME: &str = "kind,discrim,name";

    pub(crate) fn get_default_wrapped_save() -> JsValue {
        serde_wasm_bindgen::to_value(&WrappedData {
            save_name: String::from(SAVE_NAME_STR),
            save_data_kind: SaveDataKind::MainSave,
            save_data_discrim: SaveDataDiscrim::NONE,
            data: Cow::Borrowed(Bytes::new(DEFAULT_SAVE_ZLIB_CBOR)),
        })
        .expect("wrapped save data should be serializable")
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use flate2::read::ZlibDecoder;

    use crate::{consts::saves::DEFAULT_SAVE_ZLIB_CBOR, storage::save_data::UnvalidatedSaveData};

    #[test]
    fn default_save_is_valid() {
        let mut decompressed = Vec::new();
        ZlibDecoder::new(DEFAULT_SAVE_ZLIB_CBOR)
            .read_to_end(&mut decompressed)
            .expect("decompression should work");

        // DEBUG
        eprintln!("===== BEGIN CBOR =====");
        for byte in &decompressed {
            eprint!("{byte:02X}");
        }
        eprintln!("\n===== END CBOR =====");

        let data = cbor4ii::serde::from_slice::<UnvalidatedSaveData>(&decompressed)
            .expect("cbor should be in the right format");

        let _ = data.validate().expect("save data should be valid");
    }
}
