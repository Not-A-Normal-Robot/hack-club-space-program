//! An abstraction for OPFS directories in the web.

use core::fmt::Display;
use derive_more::{AsRef, Deref};
use std::sync::LazyLock;
use thiserror::Error;
use wasm_bindgen::{JsCast as _, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    FileSystemDirectoryHandle as FsDirHandle, FileSystemFileHandle as FsFileHandle,
    FileSystemGetDirectoryOptions as FsGetDirOptions, FileSystemGetFileOptions as FsGetFileOptions,
};

use crate::{
    consts::saves::{MAIN_SAVE_FILE_NAME, SAVES_DIR, STORAGE_DIR},
    storage::{SaveList, SaveName, save_data::UnvalidatedSaveData},
};

/// FS get-directory options
fn dir_opt(create: bool) -> &'static FsGetDirOptions {
    static FALSE: LazyLock<FsGetDirOptions> = LazyLock::new(|| {
        let o = FsGetDirOptions::new();
        o.set_create(false);
        o
    });

    static TRUE: LazyLock<FsGetDirOptions> = LazyLock::new(|| {
        let o = FsGetDirOptions::new();
        o.set_create(true);
        o
    });

    if create { &TRUE } else { &FALSE }
}

/// FS get-file options
fn file_opt(create: bool) -> &'static FsGetFileOptions {
    static FALSE: LazyLock<FsGetFileOptions> = LazyLock::new(|| {
        let o = FsGetFileOptions::new();
        o.set_create(false);
        o
    });

    static TRUE: LazyLock<FsGetFileOptions> = LazyLock::new(|| {
        let o = FsGetFileOptions::new();
        o.set_create(true);
        o
    });

    if create { &TRUE } else { &FALSE }
}

/// The storage directory of the game.
///
/// Location: `<STORAGE>`
#[derive(Clone, Debug, AsRef, Deref)]
pub(super) struct StorageDir(#[deref] FsDirHandle);

#[derive(Clone, Debug, Error)]
pub(crate) enum StorageDirGetterError {
    /// There is no window
    NoWindow,
    /// Error getting root dir
    RootDirError(DirGetterError),
    /// Error getting storage dir
    StorageDirError(DirGetterError),
}

impl Display for StorageDirGetterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("<StorageDirGetterError as Display>::fmt() | {self:?}");
    }
}

#[derive(Clone, Debug, Error)]
pub(crate) enum DirGetterError {
    /// Failed to get directory
    GetError(JsValue),
    /// Wrong type returned by directory getter
    GotWrongType(JsValue),
}

impl Display for DirGetterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("<DirGetterError as Display>::fmt() | {self:?}");
    }
}

#[derive(Clone, Debug, Error)]
pub(crate) enum FileGetterError {
    /// Failed to get file
    GetError(JsValue),
    /// Wrong type returned by file getter
    GotWrongType(JsValue),
}

impl Display for FileGetterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("<FileGetterError as Display>::fmt() | {self:?}");
    }
}

impl StorageDir {
    /// Gets and ensures the storage directory for the game.
    ///
    /// Also asks for persistent storage permission, but does not await
    /// it.
    pub(super) async fn new() -> Result<Self, StorageDirGetterError> {
        let storage = web_sys::window()
            .ok_or(StorageDirGetterError::NoWindow)?
            .navigator()
            .storage();
        let _ = storage.persist();
        let root = JsFuture::from(storage.get_directory())
            .await
            .map_err(|e| StorageDirGetterError::RootDirError(DirGetterError::GetError(e)))?
            .dyn_into::<FsDirHandle>()
            .map_err(|e| StorageDirGetterError::RootDirError(DirGetterError::GotWrongType(e)))?;
        let dir =
            JsFuture::from(root.get_directory_handle_with_options(STORAGE_DIR, dir_opt(true)))
                .await
                .map_err(|e| StorageDirGetterError::StorageDirError(DirGetterError::GetError(e)))?
                .dyn_into::<FsDirHandle>()
                .map_err(|e| {
                    StorageDirGetterError::StorageDirError(DirGetterError::GotWrongType(e))
                })?;

        Ok(StorageDir(dir))
    }

    /// Gets and ensures the saves directory for the game.
    pub(super) async fn saves(&self) -> Result<SavesDir, DirGetterError> {
        let promise = self
            .0
            .get_directory_handle_with_options(SAVES_DIR, dir_opt(true));

        JsFuture::from(promise)
            .await
            .map_err(DirGetterError::GetError)?
            .dyn_into::<FsDirHandle>()
            .map_err(DirGetterError::GotWrongType)
            .map(SavesDir)
    }
}

/// The saves directory of the game.
///
/// Location: `<STORAGE>/saves`
///
/// Get this struct using [`StorageDir::saves()`]
#[derive(Clone, Debug, AsRef, Deref)]
pub(super) struct SavesDir(#[deref] FsDirHandle);

impl SavesDir {
    /// Lists all the saves in this directory.
    pub(super) async fn list(&self) -> SaveList {
        todo!();
    }

    /// Gets the save directory for the save.
    ///
    /// # Parameters
    /// - `name`: The name of the save.
    /// - `ensure`: Whether or not to create the directory if it doesn't exist.
    pub(super) async fn save(
        &self,
        name: &SaveName,
        ensure: bool,
    ) -> Result<SaveDir, DirGetterError> {
        JsFuture::from(
            self.0
                .get_directory_handle_with_options(&name.0, dir_opt(ensure)),
        )
        .await
        .map_err(DirGetterError::GetError)?
        .dyn_into::<FsDirHandle>()
        .map_err(DirGetterError::GotWrongType)
        .map(SaveDir)
    }
}

/// The directory for a specific game save.
///
/// Location: `<STORAGE>/saves/<SAVE_NAME>`
///
/// Get this struct using [`SavesDir::save()`].
#[derive(Clone, Debug, AsRef, Deref)]
pub(super) struct SaveDir(#[deref] FsDirHandle);

impl SaveDir {
    /// Gets the main save file of this save.
    ///
    /// # Parameters
    /// - `ensure`: Whether or not to create the file if it doesn't exist.
    pub(super) async fn main_save(&self, ensure: bool) -> Result<MainSaveFile, FileGetterError> {
        JsFuture::from(
            self.0
                .get_file_handle_with_options(MAIN_SAVE_FILE_NAME, file_opt(ensure)),
        )
        .await
        .map_err(FileGetterError::GetError)?
        .dyn_into::<FsFileHandle>()
        .map_err(FileGetterError::GotWrongType)
        .map(MainSaveFile)
    }
}

#[derive(Clone, Debug, AsRef, Deref)]
pub(super) struct MainSaveFile(#[deref] FsFileHandle);

impl MainSaveFile {
    async fn read(&self) -> Result<UnvalidatedSaveData, JsValue> {
        todo!();
    }
}
