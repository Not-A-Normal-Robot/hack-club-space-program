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
    consts::saves::web::{
        DEFAULT_WRAPPED_SAVE, KEY_SAVE_NAME, KEY_SAVE_VALUE, SAVE_OBJECT_STORE_NAME, STORAGE_DB,
        STORAGE_DB_VERSION,
    },
    storage::{
        SaveInitError, SaveList, SaveListError, SaveName, SaveReadError, StorageImpl,
        save_data::UnvalidatedSaveData,
    },
};
use idb::{
    DatabaseEvent, Factory, ObjectStoreParams, Query, TransactionMode, event::VersionChangeEvent,
};
use serde::Serialize;
use std::sync::mpsc::SyncSender;
use wasm_bindgen::JsValue;
use web_sys::js_sys::Reflect;

fn handle_upgrade_inner(event: VersionChangeEvent) -> Result<(), SaveInitError> {
    let db = event.database().map_err(SaveInitError::UpgradeError)?;

    let mut save_params = ObjectStoreParams::new();
    save_params
        .auto_increment(false)
        .key_path(Some(idb::KeyPath::Single(String::from(KEY_SAVE_NAME))));

    db.create_object_store(SAVE_OBJECT_STORE_NAME, save_params)
        .map_err(SaveInitError::UpgradeError)?;

    Ok(())
}

fn create_upgrade_handler(tx: SyncSender<SaveInitError>) -> impl Fn(VersionChangeEvent) {
    move |event| {
        let res = handle_upgrade_inner(event);

        if let Err(e) = res {
            tx.send(e).unwrap();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WebStorage;

impl StorageImpl for WebStorage {
    async fn init_saves(self) -> Result<(), SaveInitError> {
        let mut open_req = Factory::new()
            .map_err(SaveInitError::FactoryInit)?
            .open(STORAGE_DB, Some(STORAGE_DB_VERSION))
            .map_err(SaveInitError::DbOpenRequest)?;

        let (err_tx, err_rx) = std::sync::mpsc::sync_channel(1);

        open_req.on_upgrade_needed(create_upgrade_handler(err_tx));

        let db = open_req.await.map_err(SaveInitError::DbOpen)?;

        if let Ok(err) = err_rx.try_recv() {
            return Err(err);
        }

        // TODO: Remove this when we implement saving and multi-save-files
        bevy::log::debug!("{DEFAULT_WRAPPED_SAVE}");
        let obj: serde_json::Value = serde_json::from_str(DEFAULT_WRAPPED_SAVE)
            .expect("constant `DEFAULT_WRAPPED_SAVE` should be valid json");
        bevy::log::debug!("obj: {obj}");
        let serializer = serde_wasm_bindgen::Serializer::new()
            .serialize_maps_as_objects(true)
            .serialize_large_number_types_as_bigints(true);
        let obj = obj
            .serialize(&serializer)
            .expect("json value should be serializable as js value");
        web_sys::console::debug_1(&obj);

        let trans = db
            .transaction(&[SAVE_OBJECT_STORE_NAME], TransactionMode::ReadWrite)
            .map_err(SaveInitError::DbOpen)?;
        let store = trans
            .object_store(SAVE_OBJECT_STORE_NAME)
            .map_err(SaveInitError::DbOpen)?;

        store.put(&obj, None).map_err(SaveInitError::DbOpen)?;

        trans.commit().map_err(SaveInitError::DbOpen)?;

        Ok(())
    }

    async fn get_save_list(self) -> SaveList {
        let factory = match Factory::new() {
            Ok(f) => f,
            Err(e) => return SaveListError::FactoryInit(e).into(),
        };

        let db = match factory.open(STORAGE_DB, Some(STORAGE_DB_VERSION)) {
            Ok(db) => db,
            Err(e) => {
                return SaveListError::DbOpenRequest(e).into();
            }
        };

        let db = match db.await {
            Ok(db) => db,
            Err(e) => {
                return SaveListError::DbOpen(e).into();
            }
        };

        let trans = match db.transaction(&[SAVE_OBJECT_STORE_NAME], TransactionMode::ReadOnly) {
            Ok(trans) => trans,
            Err(e) => {
                return SaveListError::TransactionRequest(e).into();
            }
        };

        let store = match trans.object_store(SAVE_OBJECT_STORE_NAME) {
            Ok(s) => s,
            Err(e) => {
                return SaveListError::ObjectStoreRequest(e).into();
            }
        };

        let req = match store.get_all(None, None) {
            Ok(r) => r,
            Err(e) => {
                return SaveListError::ObjectStoreReadRequest(e).into();
            }
        };

        let objects = match req.await {
            Ok(res) => res,
            Err(e) => {
                return SaveListError::ObjectStoreRead(e).into();
            }
        };

        let mut names: Vec<SaveName> = Vec::with_capacity(objects.len());
        let mut errors: Vec<SaveListError> = Vec::new();

        let key = JsValue::from_str(KEY_SAVE_NAME);

        for obj in objects {
            let name = match Reflect::get(&obj, &key) {
                Ok(v) => v,
                Err(e) => {
                    errors.push(SaveListError::NameExtraction(e));
                    continue;
                }
            };

            let Some(name) = name.as_string() else {
                errors.push(SaveListError::InvalidSaveName);
                continue;
            };

            names.push(name.into());
        }

        SaveList {
            saves: names.into(),
            errors: errors.into(),
        }
    }

    async fn load(self, save_name: &SaveName) -> Result<UnvalidatedSaveData, SaveReadError> {
        let trans = Factory::new()
            .map_err(SaveReadError::FactoryInit)?
            .open(STORAGE_DB, None)
            .map_err(SaveReadError::DbOpenRequest)?
            .await
            .map_err(SaveReadError::DbOpen)?
            .transaction(&[SAVE_OBJECT_STORE_NAME], TransactionMode::ReadOnly)
            .map_err(SaveReadError::TransactionRequest)?;

        let value = trans
            .object_store(SAVE_OBJECT_STORE_NAME)
            .map_err(SaveReadError::ObjectStoreRequest)?
            .get(Query::Key(JsValue::from_str(&save_name.0)))
            .map_err(SaveReadError::ObjectStoreReadRequest)?
            .await
            .map_err(SaveReadError::ObjectStoreRead)?
            .ok_or(SaveReadError::EmptyReadResult)?;

        drop(trans);

        let value = Reflect::get(&value, &JsValue::from_str(KEY_SAVE_VALUE))
            .map_err(SaveReadError::ValueExtraction)?;

        Ok(serde_wasm_bindgen::from_value::<UnvalidatedSaveData>(
            value,
        )?)
    }
}

/// This module is public because `wasm_bindgen_test` requires it to.
#[cfg(test)]
#[doc(hidden)]
pub mod _tests {
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn it_works() {
        web_sys::console::log_1(&JsValue::from_str("It works!"));
    }

    #[wasm_bindgen_test]
    fn this_doesnt() {
        panic!("oh no!");
    }
}
