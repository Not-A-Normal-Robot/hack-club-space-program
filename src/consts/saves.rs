use crate::storage::SaveName;

const SAVE_NAME_STR: &str = "demo";

// TODO: Remove this when we implement saving and multi-save-files
pub(crate) fn get_default_save_name() -> SaveName {
    #[cfg(target_family = "wasm")]
    {
        return SaveName::from(String::from(SAVE_NAME_STR));
    }

    #[cfg(not(target_family = "wasm"))]
    {
        use std::ffi::OsString;

        SaveName::from(OsString::from(SAVE_NAME_STR))
    }
}

// TODO: Remove this when we implement starting a new game
pub(crate) static DEFAULT_SAVE: &str = include_str!("./save_data.json");
