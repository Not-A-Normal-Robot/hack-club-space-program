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

use crate::{
    consts::saves::{DEFAULT_SAVE_ZSTD_CBOR, SAVE_NAME_STR},
    storage::{
        SaveInitError, SaveList, SaveListError, SaveName, SaveReadError, SaveResetError,
        StorageImpl, save_data::UnvalidatedSaveData,
    },
};

pub(crate) mod dirs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WebStorage;

impl StorageImpl for WebStorage {
    async fn init_saves(self) -> Result<(), SaveInitError> {
        let storage_dir = dirs::StorageDir::new().await?;
        let saves = storage_dir.saves().await?;

        // TODO: Remove this when we implement saving and multi-save-files
        let save = saves
            .save(&SaveName(SAVE_NAME_STR.into()), true)
            .await
            .expect("save dir creation should work");

        let main_save_file = save
            .main_save(true)
            .await
            .expect("main save file creation should work");
        main_save_file
            .write(DEFAULT_SAVE_ZSTD_CBOR)
            .await
            .expect("main save file init should work");

        Ok(())
    }

    async fn get_save_list(self) -> SaveList {
        let storage_dir = match dirs::StorageDir::new().await {
            Ok(dir) => dir,
            Err(e) => return SaveListError::StorageDirGetter(e).into(),
        };
        let saves = match storage_dir.saves().await {
            Ok(dir) => dir,
            Err(e) => return SaveListError::SavesDirGetter(e).into(),
        };
        saves.list().await
    }

    async fn load(self, save_name: &SaveName) -> Result<UnvalidatedSaveData, SaveReadError> {
        let storage_dir = dirs::StorageDir::new().await?;
        let saves = storage_dir
            .saves()
            .await
            .map_err(SaveReadError::SavesDirGetter)?;
        let save = saves
            .save(save_name, false)
            .await
            .map_err(SaveReadError::SaveDirGetter)?;
        let main_save = save.main_save(false).await?;
        Ok(main_save.read().await?)
    }

    async fn reset(self) -> Result<(), SaveResetError> {
        dirs::StorageDir::clear().await?;
        self.init_saves().await?;

        Ok(())
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
