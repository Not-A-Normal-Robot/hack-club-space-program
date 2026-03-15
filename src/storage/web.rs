use crate::storage::{SaveName, SaveReadError, save_data::UnvalidatedSaveData};

#[expect(dead_code)]
pub(super) async fn get_save_list() -> Box<[SaveName]> {
    todo!("get_save_list");
}

pub(super) async fn load(save_name: &str) -> Result<UnvalidatedSaveData, SaveReadError> {
    todo!("load");
}

pub(super) struct SaveListError {}
