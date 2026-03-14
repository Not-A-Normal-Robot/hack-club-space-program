use std::sync::LazyLock;

use crate::storage::SaveName;

const SAVE_NAME_STR: &str = "demo";

// TODO: Remove this when we implement saving and multi-save-files
pub(crate) static SAVE_NAME: LazyLock<SaveName> = LazyLock::new(|| {
    #[cfg(target_family = "wasm")]
    {
        return SaveName::from(String::from(SAVE_NAME_STR));
    }

    #[cfg(not(target_family = "wasm"))]
    {
        use std::ffi::OsString;

        SaveName::from(OsString::from(SAVE_NAME_STR))
    }
});
