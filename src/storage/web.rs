//! Web storage using IndexedDB.
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
    consts::saves::{
        DEFAULT_SAVE, SAVE_NAME_STR,
        web::{
            DEFAULT_WRAPPED_SAVE, KEY_SAVE_NAME, SAVE_OBJECT_STORE, STORAGE_DB, STORAGE_DB_VERSION,
        },
    },
    storage::{
        SaveInitError, SaveList, SaveName, SaveReadError, Storage, save_data::UnvalidatedSaveData,
    },
};
use core::fmt::Display;
use idb::{DatabaseEvent, Factory, ObjectStoreParams, TransactionMode, event::VersionChangeEvent};
use serde::Serialize;
use std::sync::mpsc::SyncSender;
use thiserror::Error;
use wasm_bindgen::JsValue;
use web_sys::js_sys::Object;

fn handle_upgrade_inner(event: VersionChangeEvent) -> Result<(), SaveInitError> {
    let db = event.database().map_err(SaveInitError::UpgradeError)?;

    let mut save_params = ObjectStoreParams::new();
    save_params
        .auto_increment(false)
        .key_path(Some(idb::KeyPath::Single(String::from(KEY_SAVE_NAME))));

    db.create_object_store(SAVE_OBJECT_STORE, save_params)
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

impl Storage for WebStorage {
    async fn init_saves(self) -> Result<(), SaveInitError> {
        let factory = Factory::new().map_err(SaveInitError::FactoryInit)?;
        let mut open_req = factory
            .open(STORAGE_DB, Some(STORAGE_DB_VERSION))
            .map_err(SaveInitError::DbOpenRequest)?;

        let (err_tx, err_rx) = std::sync::mpsc::sync_channel(1);

        open_req.on_upgrade_needed(create_upgrade_handler(err_tx));

        let mut db = open_req.await.map_err(SaveInitError::DbOpen)?;

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
            .transaction(&[SAVE_OBJECT_STORE], TransactionMode::ReadWrite)
            .map_err(SaveInitError::DbOpen)?;
        let store = trans
            .object_store(SAVE_OBJECT_STORE)
            .map_err(SaveInitError::DbOpen)?;

        store.put(&obj, None).map_err(SaveInitError::DbOpen)?;

        Ok(())
    }

    async fn get_save_list(self) -> SaveList {
        todo!("web::get_save_list");
    }

    async fn load(self, _save_name: &SaveName) -> Result<UnvalidatedSaveData, SaveReadError> {
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
