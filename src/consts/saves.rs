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

/// The storage directory, relative to `dirs::data_dir()`.
pub(crate) static STORAGE_DIR: &str = "hack-club-space-program";

/// The saves directory, relative to the storage directory.
///
/// Contains many directories, each for a specific save.
pub(crate) static SAVES_DIR: &str = "saves";

/// The main save file's name, relative to the specific save's directory.
pub(crate) static MAIN_SAVE_FILE_NAME: &str = "main.cbor.zlib";

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
