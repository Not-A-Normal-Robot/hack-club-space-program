use crate::storage::{SaveName, SaveReadError, save_data::UnvalidatedSaveData};

#[expect(dead_code)]
pub(super) fn get_save_list() -> Box<[SaveName]> {
    todo!("get_save_list");
}

pub(super) fn load(save_name: &str) -> Result<UnvalidatedSaveData, SaveReadError> {
    todo!("load");
}
