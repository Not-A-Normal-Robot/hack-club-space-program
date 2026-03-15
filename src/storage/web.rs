use crate::storage::{SaveList, SaveName, SaveReadError, save_data::UnvalidatedSaveData};
use core::fmt::Display;
use thiserror::Error;

#[expect(dead_code)]
pub(super) fn get_save_list() -> SaveList {
    todo!("get_save_list");
}

pub(super) async fn load(save_name: &str) -> Result<UnvalidatedSaveData, SaveReadError> {
    todo!("load");
}

#[derive(Debug, Error)]
pub(super) struct SaveListError {}

impl Display for SaveListError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }
}
