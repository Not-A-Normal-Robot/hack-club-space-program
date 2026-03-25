use std::{
    ffi::OsString,
    fs::{self, File},
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
};

use flate2::read::ZlibDecoder;

use crate::{
    consts::saves::{DEFAULT_SAVE_ZLIB_CBOR, SAVE_NAME_STR},
    storage::{
        SaveInitError, SaveList, SaveListError, SaveName, SaveReadError, SaveResetError,
        StorageImpl,
        nonweb::{
            dirs::{DirAbstraction, StorageDir},
            risk::check_path_risk,
        },
        save_data::UnvalidatedSaveData,
    },
};

pub(crate) mod dirs;
pub(crate) mod risk;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NonWebStorage;

impl StorageImpl for NonWebStorage {
    async fn init_saves(self) -> Result<(), SaveInitError> {
        let dir = StorageDir::new().ok_or(SaveInitError::NoSaveDir)?;
        let saves = dir.saves();
        saves.ensure().map_err(SaveInitError::DirCreation)?;

        let save = saves.get_save(&SAVE_NAME_STR.to_string().into());
        save.ensure().map_err(SaveInitError::DirCreation)?;
        let main_save = save.main_save();
        fs::write(main_save, DEFAULT_SAVE_ZLIB_CBOR).map_err(SaveInitError::DirCreation)
    }

    async fn get_save_list(self) -> SaveList {
        let Some(dir) = StorageDir::new() else {
            return SaveListError::NoSaveDir.into();
        };
        let saves = dir.saves();
        saves.list_saves()
    }

    async fn load(self, save_name: &SaveName) -> Result<UnvalidatedSaveData, SaveReadError> {
        let storage = StorageDir::new().ok_or(SaveReadError::NoSaveDir)?;
        let saves = storage.saves();
        let save = saves.get_save(save_name);
        let savefile_path = save.main_save();
        let savefile = File::open(savefile_path)?;

        let decoder = ZlibDecoder::new(savefile);
        let buf_decoder = BufReader::new(decoder);

        Ok(cbor4ii::serde::from_reader::<
            UnvalidatedSaveData,
            BufReader<ZlibDecoder<File>>,
        >(buf_decoder)?)
    }

    async fn reset(self) -> Result<(), SaveResetError> {
        let storage = StorageDir::new().ok_or(SaveResetError::NoSaveDir)?;
        let dir = storage.get_path();

        if let Err(e) = check_path_risk(&dir) {
            return Err(SaveResetError::RiskyPath {
                path: dir.to_path_buf(),
                reason: e,
            });
        }

        fs::remove_dir_all(dir)?;

        Ok(self.init_saves().await?)
    }
}

#[cfg(test)]
mod tests {
    use smol_macros::test;

    // Things are split into inner and tester fns for
    // better intellisense (r-a doesn't quite like decl macros)

    async fn test_saves_inner() {
        let storage = crate::storage::get_storage();
        storage.init_saves().await.unwrap();
        storage.reset().await.unwrap();
        let res = storage.get_save_list().await;
        assert_eq!(res.errors.len(), 0, "Save listing errored: {}", res.errors);

        // TODO: Remove this when we implement saving and multi-save-files
        assert_eq!(res.saves.len(), 1);
        let save_name = res.saves.first().unwrap();
        let save_data = storage.load(save_name).await.unwrap();
        save_data.validate().unwrap();
    }

    test! {
        async fn test_saves() {
            test_saves_inner().await;
        }
    }
}
