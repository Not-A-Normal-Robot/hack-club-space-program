use bevy::{
    log::warn, platform::collections::HashSet, platform::time::Instant,
    tasks::futures_lite::future::yield_now,
};
use core::{
    fmt::{Debug, Display},
    sync::atomic::{AtomicU8, Ordering},
};
use derive_more::{Deref, DerefMut};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{borrow::Cow, io, sync::Mutex};
#[cfg(not(target_family = "wasm"))]
use std::{ffi::OsString, path::PathBuf};
use thiserror::Error;

#[cfg(not(target_family = "wasm"))]
use crate::storage::nonweb::risk::RiskyPathReason;
use crate::{
    consts::saves::INIT_TIMEOUT,
    fl,
    storage::save_data::{SaveDataError, UnvalidatedSaveData},
};

pub(crate) mod save_data;

#[cfg(not(target_family = "wasm"))]
mod nonweb;
/// This module is public because `wasm_bindgen_test` requires it to.
#[cfg(target_family = "wasm")]
pub mod web;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum InitStatus {
    NotInitialized = 0,
    Initialized = 1,
    Failed = 2,
}

impl InitStatus {
    const fn discriminant(self) -> u8 {
        self as u8
    }
}

/// Uses the discriminants for [`InitStatus`].
static STORAGE_INITIALIZATION_STATUS: AtomicU8 =
    AtomicU8::new(InitStatus::NotInitialized.discriminant());

#[derive(Clone, Copy, Debug)]
pub(crate) struct Storage(
    #[cfg(target_family = "wasm")] web::WebStorage,
    #[cfg(not(target_family = "wasm"))] nonweb::NonWebStorage,
);

#[derive(Clone, Copy, Debug, Error)]
pub(crate) enum StorageNotInitialized {
    /// Save initialization took too long
    TimedOut,
    /// Save initialization errored out
    InitError,
}

impl Display for StorageNotInitialized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.write_str(&fl!("error__saveGeneral__initTimeout"))
        match self {
            Self::TimedOut => f.write_str(&fl!("error__saveGeneral__initTimeout")),
            Self::InitError => f.write_str(&fl!("error__saveGeneral__initFailed")),
        }
    }
}

impl Storage {
    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    pub(crate) async fn init_saves(self) -> Result<(), SaveInitError> {
        let res = self.0.init_saves().await;

        let init_status = match &res {
            Ok(()) => InitStatus::Initialized,
            Err(_) => InitStatus::Failed,
        };
        STORAGE_INITIALIZATION_STATUS.store(init_status.discriminant(), Ordering::Relaxed);

        res
    }

    /// Wait for storage initialization to successfully finish.
    pub(crate) async fn await_save_init(self) -> Result<(), StorageNotInitialized> {
        let start = Instant::now();
        let timeout_end = start + INIT_TIMEOUT;

        loop {
            let state = STORAGE_INITIALIZATION_STATUS.load(Ordering::Relaxed);
            if state == InitStatus::Failed.discriminant() {
                return Err(StorageNotInitialized::InitError);
            } else if state == InitStatus::Initialized.discriminant() {
                return Ok(());
            }

            if timeout_end < Instant::now() {
                return Err(StorageNotInitialized::TimedOut);
            }

            yield_now().await;
        }
    }

    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    // async fn get_save_list(&self) -> SaveList;
    pub(crate) async fn get_save_list(self) -> SaveList {
        if let Err(e) = self.await_save_init().await {
            return SaveListError::StorageNotInitialized(e).into();
        }

        self.0.get_save_list().await
    }

    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    pub(crate) async fn load(
        self,
        save_name: &SaveName,
    ) -> Result<UnvalidatedSaveData, SaveReadError> {
        self.await_save_init().await?;

        self.0.load(save_name).await
    }

    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    pub(crate) async fn reset(self) -> Result<(), SaveResetError> {
        match self.await_save_init().await {
            Ok(()) | Err(StorageNotInitialized::InitError) => STORAGE_INITIALIZATION_STATUS
                .store(InitStatus::NotInitialized.discriminant(), Ordering::Relaxed),
            Err(StorageNotInitialized::TimedOut) => return Err(SaveResetError::StorageInitTimeout),
        };

        let res = self.0.reset().await;
        let new_status = if res.is_err() {
            InitStatus::Failed
        } else {
            InitStatus::Initialized
        };

        STORAGE_INITIALIZATION_STATUS.store(new_status.discriminant(), Ordering::Relaxed);

        res
    }
}

trait StorageImpl: Copy + Sized + Send + Sync {
    /// This function is not to be overridden.
    #[doc(hidden)]
    #[deprecated = "This function is not to be ran."]
    #[inline(never)]
    #[cold]
    fn __const_checks() -> core::convert::Infallible {
        const { assert!(core::mem::size_of::<Self>() == 0) };
        unreachable!();
    }

    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    #[cfg(not(target_family = "wasm"))]
    fn init_saves(self) -> impl Future<Output = Result<(), SaveInitError>> + Send;

    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    #[cfg(target_family = "wasm")]
    async fn init_saves(self) -> Result<(), SaveInitError>;

    /// Gets the save list.
    ///
    /// # Initialized
    /// You may assume that the save subsystem has been initialized.
    ///
    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    // async fn get_save_list(&self) -> SaveList;
    #[cfg(not(target_family = "wasm"))]
    fn get_save_list(self) -> impl Future<Output = SaveList> + Send;

    /// Gets the save list.
    ///
    /// # Initialized
    /// You may assume that the save subsystem has been initialized.
    ///
    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    // async fn get_save_list(&self) -> SaveList;
    #[cfg(target_family = "wasm")]
    async fn get_save_list(self) -> SaveList;

    /// Loads a save.
    ///
    /// # Initialized
    /// You may assume that the save subsystem has been initialized.
    ///
    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    #[cfg(not(target_family = "wasm"))]
    fn load(
        self,
        save_name: &SaveName,
    ) -> impl Future<Output = Result<UnvalidatedSaveData, SaveReadError>> + Send;

    /// Loads a save.
    ///
    /// # Initialized
    /// You may assume that the save subsystem has been initialized.
    ///
    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    #[cfg(target_family = "wasm")]
    async fn load(self, save_name: &SaveName) -> Result<UnvalidatedSaveData, SaveReadError>;

    /// Resets the storage subsystem to its initial state.
    ///
    /// # Initialized or Failed
    /// You may assume that the save subsystem's attempts to initialize
    /// have finished.
    /// This does NOT guarantee that the initialization SUCCEEDED; it
    /// could have also failed.
    #[cfg(not(target_family = "wasm"))]
    fn reset(self) -> impl Future<Output = Result<(), SaveResetError>> + Send;

    /// Resets the storage subsystem to its initial state.
    ///
    /// # Initialized or Failed
    /// You may assume that the save subsystem's attempts to initialize
    /// have finished.
    /// This does NOT guarantee that the initialization SUCCEEDED; it
    /// could have also failed.
    #[cfg(target_family = "wasm")]
    async fn reset(self) -> Result<(), SaveResetError>;
}

pub(crate) fn get_storage() -> Storage {
    #[cfg(target_family = "wasm")]
    {
        Storage(web::WebStorage)
    }

    #[cfg(not(target_family = "wasm"))]
    {
        Storage(nonweb::NonWebStorage)
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum SaveInitError {
    #[cfg(not(target_family = "wasm"))]
    NoSaveDir,
    #[cfg(not(target_family = "wasm"))]
    DirCreation(io::Error),
    /// Something went wrong while trying to initialize the idb
    /// factory
    #[cfg(target_family = "wasm")]
    FactoryInit(idb::Error),
    /// Something went wrong while requesting the db to be opened
    #[cfg(target_family = "wasm")]
    DbOpenRequest(idb::Error),
    /// Something went wrong while opening the db
    #[cfg(target_family = "wasm")]
    DbOpen(idb::Error),
    /// Something went wrong while initializing or
    /// upgrading the db
    #[cfg(target_family = "wasm")]
    UpgradeError(idb::Error),
}

impl Display for SaveInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(not(target_family = "wasm"))]
            Self::NoSaveDir => f.write_str(&fl!("error__saveGeneral__noSaveDir")),
            #[cfg(not(target_family = "wasm"))]
            Self::DirCreation(inner) => f.write_str(&fl!(
                "error__saveInit__dirCreation",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::FactoryInit(inner) => f.write_str(&fl!(
                "error__saveGeneral__factoryInit",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::DbOpenRequest(inner) => f.write_str(&fl!(
                "error__saveGeneral__dbOpenRequest",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::DbOpen(inner) => f.write_str(&fl!(
                "error__saveGeneral__dbOpen",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::UpgradeError(inner) => f.write_str(&fl!(
                "error__saveInit__upgradeError",
                inner = inner.to_string()
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SaveName(
    #[cfg(not(target_family = "wasm"))] OsString,
    #[cfg(target_family = "wasm")] String,
);

#[cfg(not(target_family = "wasm"))]
impl From<OsString> for SaveName {
    fn from(value: OsString) -> Self {
        Self(value)
    }
}

impl From<String> for SaveName {
    fn from(value: String) -> Self {
        #[cfg(target_family = "wasm")]
        {
            Self(value)
        }

        #[cfg(not(target_family = "wasm"))]
        {
            Self(value.into())
        }
    }
}

impl SaveName {
    pub(crate) fn to_str(&self) -> Cow<'_, str> {
        #[cfg(target_family = "wasm")]
        {
            Cow::Borrowed(&self.0)
        }

        #[cfg(not(target_family = "wasm"))]
        {
            self.0.to_string_lossy()
        }
    }
}

impl Display for SaveName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_str())
    }
}

#[derive(Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub(crate) enum SaveDataKind {
    MainSave = 0,
    QuickSave = 1,
}

#[derive(Debug)]
pub(crate) struct SaveList {
    pub(crate) saves: Box<[SaveName]>,
    pub(crate) errors: SaveListErrors,
}

impl From<SaveListError> for SaveList {
    fn from(value: SaveListError) -> Self {
        Self {
            saves: Box::from([]),
            errors: value.into(),
        }
    }
}

#[derive(Debug, Error, Deref, DerefMut)]
pub(crate) struct SaveListErrors(pub(crate) Box<[SaveListError]>);

impl From<Box<[SaveListError]>> for SaveListErrors {
    fn from(value: Box<[SaveListError]>) -> Self {
        Self(value)
    }
}

impl From<Vec<SaveListError>> for SaveListErrors {
    fn from(value: Vec<SaveListError>) -> Self {
        Self(value.into_boxed_slice())
    }
}

impl From<SaveListError> for SaveListErrors {
    fn from(value: SaveListError) -> Self {
        Self(Box::from([value]))
    }
}

impl SaveListErrors {
    #[cold]
    fn warn_ui_once(location: &core::panic::Location) {
        static UI_CALLER_SET: Mutex<HashSet<String>> = Mutex::new(HashSet::new());

        let first_call = UI_CALLER_SET.try_lock().is_ok_and(|mut set| {
            let file = location.file();
            if set.contains(file) {
                return false;
            }

            set.insert(file.to_owned())
        });

        if first_call {
            warn!(
                "<{} as Display>::fmt called from {location};
                for UI, please traverse the list manually and make
                a bullet list instead of relying on the Display
                impl, because this Display impl is not localized",
                core::any::type_name::<Self>()
            );
        }
    }
}

impl Display for SaveListErrors {
    #[track_caller]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        /// Strings for detecting whether or not something is for UI.
        const UI_STRINGS: [&str; 2] = ["/ui/", "/ui.rs"];

        let location = core::panic::Location::caller();
        let file = location.file();

        if UI_STRINGS.iter().any(|s| file.contains(s)) {
            Self::warn_ui_once(location);
        }

        for error in &self.0 {
            writeln!(f, "- {error}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub(crate) enum SaveListError {
    /// Storage wasn't initialized properly
    StorageNotInitialized(#[from] StorageNotInitialized),
    /// Couldn't decide on a save dir
    #[cfg(not(target_family = "wasm"))]
    NoSaveDir,
    /// Couldn't read the save dir
    #[cfg(not(target_family = "wasm"))]
    ReadDirError(io::Error),
    /// Couldn't read a save dir entry
    #[cfg(not(target_family = "wasm"))]
    DirEntryError(io::Error),
    /// Couldn't read an entry's file type
    #[cfg(not(target_family = "wasm"))]
    FileTypeError { path: PathBuf, error: io::Error },
    /// Dir entry isn't a directory
    #[cfg(not(target_family = "wasm"))]
    NotADir(PathBuf),
    /// Something went wrong while trying to initialize the idb
    /// factory
    #[cfg(target_family = "wasm")]
    FactoryInit(idb::Error),
    /// Something went wrong while requesting the db to be opened
    #[cfg(target_family = "wasm")]
    DbOpenRequest(idb::Error),
    /// Something went wrong while opening the db
    #[cfg(target_family = "wasm")]
    DbOpen(idb::Error),
    /// Something went wrong requesting a transaction
    #[cfg(target_family = "wasm")]
    TransactionRequest(idb::Error),
    /// Something went wrong requesting the object store
    #[cfg(target_family = "wasm")]
    ObjectStoreRequest(idb::Error),
    /// Something went wrong requesting a read from the object store
    #[cfg(target_family = "wasm")]
    ObjectStoreReadRequest(idb::Error),
    /// Something went wrong reading from the object store
    #[cfg(target_family = "wasm")]
    ObjectStoreRead(idb::Error),
    /// Something went wrong extracting the save name key.
    #[cfg(target_family = "wasm")]
    NameExtraction(wasm_bindgen::JsValue),
    /// The save name key is not valid UTF-8
    #[cfg(target_family = "wasm")]
    InvalidSaveName,
}
// pub(crate) struct SaveListError(SaveListErrorInner);

impl Display for SaveListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StorageNotInitialized(inner) => <StorageNotInitialized as Display>::fmt(inner, f),
            #[cfg(not(target_family = "wasm"))]
            Self::NoSaveDir => f.write_str(&fl!("error__saveGeneral__noSaveDir")),
            #[cfg(not(target_family = "wasm"))]
            Self::ReadDirError(inner) => {
                f.write_str(&fl!("error__saveList__readDir", inner = inner.to_string()))
            }
            #[cfg(not(target_family = "wasm"))]
            Self::DirEntryError(inner) => {
                f.write_str(&fl!("error__saveList__dirEntry", inner = inner.to_string()))
            }
            #[cfg(not(target_family = "wasm"))]
            Self::FileTypeError { path, error } => f.write_str(&fl!(
                "error__saveList__fileType",
                path = path.to_string_lossy(),
                inner = error.to_string()
            )),
            #[cfg(not(target_family = "wasm"))]
            Self::NotADir(path) => f.write_str(&fl!(
                "error__saveList__notDir",
                path = path.to_string_lossy()
            )),
            #[cfg(target_family = "wasm")]
            Self::FactoryInit(inner) => f.write_str(&fl!(
                "error__saveGeneral__factoryInit",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::DbOpenRequest(inner) => f.write_str(&fl!(
                "error__saveGeneral__dbOpenRequest",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::DbOpen(inner) => f.write_str(&fl!(
                "error__saveGeneral__dbOpen",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::TransactionRequest(inner) => f.write_str(&fl!(
                "error__saveGeneral__transactionRequest",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::ObjectStoreRequest(inner) => f.write_str(&fl!(
                "error__saveGeneral__objectStoreRequest",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::ObjectStoreReadRequest(inner) => f.write_str(&fl!(
                "error__saveGeneral__objectStoreReadRequest",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::ObjectStoreRead(inner) => f.write_str(&fl!(
                "error__saveGeneral__objectStoreRead",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::NameExtraction(inner) => f.write_str(&fl!(
                "error__saveList__nameExtraction",
                inner = format!("{inner:?}"),
            )),
            #[cfg(target_family = "wasm")]
            Self::InvalidSaveName => f.write_str(&fl!("error__saveList__invalidSaveName")),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum SaveReadError {
    StorageNotInitialized(#[from] StorageNotInitialized),
    #[cfg(not(target_family = "wasm"))]
    NoSaveDir,
    IoError(#[from] io::Error),
    #[cfg(not(target_family = "wasm"))]
    ParseError(#[from] cbor4ii::serde::DecodeError<std::io::Error>),
    /// Something went wrong while trying to initialize the idb
    /// factory
    #[cfg(target_family = "wasm")]
    FactoryInit(idb::Error),
    /// Something went wrong while requesting the db to be opened
    #[cfg(target_family = "wasm")]
    DbOpenRequest(idb::Error),
    /// Something went wrong while opening the db
    #[cfg(target_family = "wasm")]
    DbOpen(idb::Error),
    /// Something went wrong requesting a transaction
    #[cfg(target_family = "wasm")]
    TransactionRequest(idb::Error),
    /// Something went wrong requesting the object store
    #[cfg(target_family = "wasm")]
    ObjectStoreRequest(idb::Error),
    /// Something went wrong requesting a read from the object store
    #[cfg(target_family = "wasm")]
    ObjectStoreReadRequest(idb::Error),
    /// Something went wrong reading from the object store
    #[cfg(target_family = "wasm")]
    ObjectStoreRead(idb::Error),
    /// The object store read resulted in nothing
    #[cfg(target_family = "wasm")]
    EmptyReadResult,
    /// Something went wrong extracting the save data from the object.
    #[cfg(target_family = "wasm")]
    ValueExtraction(wasm_bindgen::JsValue),
    /// The save data is not of the correct type.
    /// (Expects Uint8Array)
    #[cfg(target_family = "wasm")]
    ValueWrongType(wasm_bindgen::JsValue),
    /// Something went wrong parsing the jsvalue
    #[cfg(target_family = "wasm")]
    ParseError(#[from] cbor4ii::serde::DecodeError<core::convert::Infallible>),
    InvalidState(#[from] SaveDataError),
}

impl Display for SaveReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StorageNotInitialized(inner) => <StorageNotInitialized as Display>::fmt(inner, f),
            #[cfg(not(target_family = "wasm"))]
            Self::NoSaveDir => f.write_str(&fl!("error__saveGeneral__noSaveDir")),
            #[cfg(not(target_family = "wasm"))]
            Self::IoError(e) => {
                f.write_str(&fl!("error__saveRead__ioError", inner = format!("{e}")))
            }
            #[cfg(target_family = "wasm")]
            Self::FactoryInit(inner) => f.write_str(&fl!(
                "error__saveGeneral__factoryInit",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::DbOpenRequest(inner) => f.write_str(&fl!(
                "error__saveGeneral__dbOpenRequest",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::DbOpen(inner) => f.write_str(&fl!(
                "error__saveGeneral__dbOpen",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::TransactionRequest(inner) => f.write_str(&fl!(
                "error__saveGeneral__transactionRequest",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::ObjectStoreRequest(inner) => f.write_str(&fl!(
                "error__saveGeneral__objectStoreRequest",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::ObjectStoreReadRequest(inner) => f.write_str(&fl!(
                "error__saveGeneral__objectStoreReadRequest",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::ObjectStoreRead(inner) => f.write_str(&fl!(
                "error__saveGeneral__objectStoreRead",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::EmptyReadResult => f.write_str(&fl!("error__saveRead__emptyRead")),
            #[cfg(target_family = "wasm")]
            Self::ValueExtraction(inner) => f.write_str(&fl!(
                "error__saveRead__valueExtraction",
                inner = format!("{inner:?}")
            )),
            Self::InvalidState(e) => <SaveDataError as Display>::fmt(e, f),
            Self::ParseError(error) => f.write_str(&fl!(
                "error__saveRead__parseError",
                inner = format!("{error}")
            )),
            _ => todo!("error l10n"),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum SaveResetError {
    /// Storage initialization attempts hasn't finished in time
    StorageInitTimeout,
    /// We couldn't decide on a save directory
    #[cfg(not(target_family = "wasm"))]
    NoSaveDir,
    /// Unwilling to remove risky directory
    #[cfg(not(target_family = "wasm"))]
    RiskyPath {
        path: PathBuf,
        reason: RiskyPathReason,
    },
    /// Failed to delete save dir
    #[cfg(not(target_family = "wasm"))]
    DeleteError(#[from] io::Error),
    /// Something went wrong while trying to initialize the idb
    /// factory
    #[cfg(target_family = "wasm")]
    FactoryInit(idb::Error),
    /// Something went wrong while requesting db deletion
    #[cfg(target_family = "wasm")]
    DbDeleteRequest(idb::Error),
    /// Something went wrong while deleting db
    #[cfg(target_family = "wasm")]
    DbDelete(idb::Error),
    InitError(#[from] SaveInitError),
}

impl Display for SaveResetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StorageInitTimeout => f.write_str(&fl!("error__saveGeneral__initTimeout")),

            #[cfg(not(target_family = "wasm"))]
            Self::NoSaveDir => f.write_str(&fl!("error__saveGeneral__noSaveDir")),
            #[cfg(not(target_family = "wasm"))]
            Self::RiskyPath { path, reason } => f.write_str(&fl!(
                "error__saveReset__riskyPath",
                path = path.to_string_lossy(),
                reason = reason.to_string()
            )),
            #[cfg(not(target_family = "wasm"))]
            Self::DeleteError(inner) => f.write_str(&fl!(
                "error__saveReset__deleteError",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::FactoryInit(inner) => f.write_str(&fl!(
                "error__saveGeneral__factoryInit",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::DbDeleteRequest(inner) => f.write_str(&fl!(
                "error__saveReset__dbDeleteRequest",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::DbDelete(inner) => f.write_str(&fl!(
                "error__saveReset__dbDelete",
                inner = inner.to_string()
            )),
            Self::InitError(inner) => f.write_str(&fl!(
                "error__saveReset__initError",
                inner = inner.to_string()
            )),
        }
    }
}
