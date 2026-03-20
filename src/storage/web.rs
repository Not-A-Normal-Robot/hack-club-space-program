use crate::storage::{
    SaveInitError, SaveList, SaveName, SaveReadError, Storage, save_data::UnvalidatedSaveData,
};
use core::fmt::Display;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WebStorage;

impl Storage for WebStorage {
    async fn init_saves(self) -> Result<(), SaveInitError> {
        todo!("web::init_saves");
    }

    async fn get_save_list(self) -> SaveList {
        todo!("web::get_save_list");
    }

    async fn load(self, save_name: &SaveName) -> Result<UnvalidatedSaveData, SaveReadError> {
        todo!("web::load");
    }
}

#[derive(Debug, Error)]
pub(super) struct SaveListError {}

impl Display for SaveListError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }
}
