//! Web storage using `IndexedDB`.
//!
//! We create a database and an object store for saves.
//! The web save wrapper schema looks like this:
//! ```json5
//! {
//!     // This name is used as a primary key
//!     "name": "my save name",
//!     // The inner save data uses the regular
//!     // save data schema used in non-web platforms.
//!     "data": { ... }
//! }
//! ```
//!
//! For more information on the save data schema, see
//! `src/consts/save_data.schema.json`.

use crate::storage::{
    SaveDataKind, SaveInitError, SaveList, SaveListError, SaveName, SaveReadError, SaveResetError,
    StorageImpl, save_data::UnvalidatedSaveData,
};

mod dirs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WebStorage;

impl StorageImpl for WebStorage {
    async fn init_saves(self) -> Result<(), SaveInitError> {
        // TODO: init saves
        // TODO: Ask for persistent data
        todo!();
    }

    async fn get_save_list(self) -> SaveList {
        todo!();
    }

    async fn load(self, save_name: &SaveName) -> Result<UnvalidatedSaveData, SaveReadError> {
        todo!();
    }

    async fn reset(self) -> Result<(), SaveResetError> {
        todo!();
    }
}

/// This module is public because `wasm_bindgen_test` requires it to.
#[cfg(test)]
#[doc(hidden)]
pub mod _tests {
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_storage() {
        let storage = crate::storage::get_storage();
        let _ = storage.init_saves().await;
        storage.reset().await.unwrap();
        let res = storage.get_save_list().await;
        assert_eq!(res.errors.len(), 0);

        // TODO: Remove this when we implement saving and multi-save-files
        assert_eq!(res.saves.len(), 1);
        let save_name = res.saves.first().unwrap();
        let save_data = storage.load(save_name).await.unwrap();
        save_data.validate().unwrap();
    }
}
