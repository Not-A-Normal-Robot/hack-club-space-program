use crate::storage::{
    SaveInitError, SaveList, SaveName, SaveReadError, save_data::UnvalidatedSaveData,
};
use core::fmt::Display;
use thiserror::Error;

pub(super) async fn init_saves() -> Result<(), SaveInitError> {
    todo!("web::init_saves");
}

#[expect(dead_code)]
pub(super) fn get_save_list() -> SaveList {
    todo!("web::get_save_list");
}

pub(super) async fn load(save_name: &str) -> Result<UnvalidatedSaveData, SaveReadError> {
    todo!("web::load");
}

#[derive(Debug, Error)]
pub(super) struct SaveListError {}

impl Display for SaveListError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }
}
