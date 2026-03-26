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
        INDEX_KIND_DISCRIM_NAME, INDEX_NAME_KIND, INDEX_NAME_ONLY, KEY_SAVE_DISCRIM, KEY_SAVE_KIND,
        KEY_SAVE_NAME, KEY_SAVE_VALUE, SAVE_OBJECT_STORE_NAME, STORAGE_DB, STORAGE_DB_VERSION,
        get_default_wrapped_save,
    },
    storage::{
        SaveDataKind, SaveInitError, SaveList, SaveListError, SaveName, SaveReadError,
        SaveResetError, StorageImpl, save_data::UnvalidatedSaveData,
    },
};
use flate2::read::ZlibDecoder;
use idb::{
    DatabaseEvent, Factory, IndexParams, KeyPath, KeyRange, ObjectStoreParams, Query,
    TransactionMode, event::VersionChangeEvent,
};
use serde::{Deserialize, Serialize, de::Visitor};
use std::{borrow::Cow, io::Read, sync::mpsc::SyncSender};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::{Array, JsString, Reflect, Uint8Array};

fn handle_upgrade_inner(event: VersionChangeEvent) -> Result<(), SaveInitError> {
    let db = event.database().map_err(SaveInitError::UpgradeError)?;

    let mut save_params = ObjectStoreParams::new();
    save_params
        .auto_increment(false)
        .key_path(Some(KeyPath::Array(vec![
            KEY_SAVE_NAME.into(),
            KEY_SAVE_KIND.into(),
            KEY_SAVE_DISCRIM.into(),
        ])));

    let store = db
        .create_object_store(SAVE_OBJECT_STORE_NAME, save_params)
        .map_err(SaveInitError::UpgradeError)?;

    store
        .create_index(INDEX_NAME_ONLY, KeyPath::Single(KEY_SAVE_NAME.into()), None)
        .map_err(SaveInitError::UpgradeError)?;

    store
        .create_index(
            INDEX_KIND_DISCRIM_NAME,
            KeyPath::Array(vec![
                KEY_SAVE_KIND.into(),
                KEY_SAVE_DISCRIM.into(),
                KEY_SAVE_NAME.into(),
            ]),
            None,
        )
        .map_err(SaveInitError::UpgradeError)?;

    store
        .create_index(
            INDEX_NAME_KIND,
            KeyPath::Array(vec![KEY_SAVE_NAME.into(), KEY_SAVE_KIND.into()]),
            None,
        )
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
        let default_wrapped_save = get_default_wrapped_save();
        bevy::log::debug!("{default_wrapped_save:?}");

        let trans = db
            .transaction(&[SAVE_OBJECT_STORE_NAME], TransactionMode::ReadWrite)
            .map_err(SaveInitError::DbOpen)?;
        let store = trans
            .object_store(SAVE_OBJECT_STORE_NAME)
            .map_err(SaveInitError::DbOpen)?;

        store
            .put(&default_wrapped_save, None)
            .map_err(SaveInitError::DbOpen)?;

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

        let index = match store.index(INDEX_KIND_DISCRIM_NAME) {
            Ok(i) => i,
            Err(e) => {
                todo!("index not found error handling | {e}");
            }
        };

        let range = {
            let array = Array::new();
            array.push_many(&[JsValue::from_str(""), JsValue::from_f64(0.0)]);
            let array = JsValue::from(array);
            KeyRange::only(&array)
        };

        let query = match range {
            Ok(range) => Some(Query::KeyRange(range)),
            Err(e) => todo!("invalid range error handling | {e}"),
        };

        let req = match index.get_all_keys(query, None) {
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

        let arr = Reflect::get(&value, &JsValue::from_str(KEY_SAVE_VALUE))
            .map_err(SaveReadError::ValueExtraction)?
            .dyn_into::<Uint8Array>()
            .map_err(SaveReadError::ValueWrongType)?;

        let mut uninit: Box<[std::mem::MaybeUninit<u8>]> =
            Box::new_uninit_slice(arr.byte_length() as usize);
        let compressed = arr.copy_to_uninit(&mut uninit);

        let mut decompressed = Vec::new();
        let mut decoder = ZlibDecoder::new(compressed as &[u8]);
        decoder.read_to_end(&mut decompressed)?;

        drop(uninit);
        drop(arr);

        Ok(cbor4ii::serde::from_slice(&decompressed)?)
    }

    async fn reset(self) -> Result<(), SaveResetError> {
        Factory::new()
            .map_err(SaveResetError::FactoryInit)?
            .delete(STORAGE_DB)
            .map_err(SaveResetError::DbDeleteRequest)?
            .await
            .map_err(SaveResetError::DbDelete)?;

        Ok(self.init_saves().await?)
    }
}

/// Data that's been wrapped for `IndexedDB`.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct WrappedData {
    #[serde(rename = "name")]
    pub(crate) save_name: String,
    #[serde(rename = "kind")]
    pub(crate) save_data_kind: SaveDataKind,
    #[serde(rename = "discrim")]
    pub(crate) save_data_discrim: SaveDataDiscrim,
    /// Zlib-compressed CBOR-encoded save data.
    pub(crate) data: Cow<'static, [u8]>,
}

/// A discriminator for different save objects of the same kind and
/// belonging to the same save.
///
/// # Serialization
/// This gets serialized as a hex string up to
/// 32 chars in length. However, the zero case is
/// special and is serialized as an empty string.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct SaveDataDiscrim(pub u128);

impl SaveDataDiscrim {
    /// No discriminator.
    ///
    /// In `IndexedDB`, this will get serialized as an empty string.
    pub(crate) const NONE: Self = Self(0);
}

impl Serialize for SaveDataDiscrim {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let num = self.0;
        if num == 0 {
            serializer.serialize_str("")
        } else {
            serializer.serialize_str(&format!("{num:x}"))
        }
    }
}

impl<'de> Deserialize<'de> for SaveDataDiscrim {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DiscrimVisitor;

        impl Visitor<'_> for DiscrimVisitor {
            type Value = SaveDataDiscrim;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#""[0-9a-fA-F]{0,32}""#)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.is_empty() {
                    Ok(SaveDataDiscrim(0))
                } else {
                    u128::from_str_radix(v, 16)
                        .map(SaveDataDiscrim)
                        .map_err(E::custom)
                }
            }
        }

        deserializer.deserialize_str(DiscrimVisitor)
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
