//! An abstraction for OPFS directories in the web.

use bevy::tasks::futures_lite::StreamExt;
use cbor4ii::serde::DecodeError;
use core::{convert::Infallible, fmt::Display, mem::MaybeUninit};
use derive_more::{AsRef, Deref};
use flate2::write::ZlibDecoder;
use std::{
    io::{self, Write},
    sync::LazyLock,
};
use thiserror::Error;
use wasm_bindgen::{JsCast as _, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_streams::ReadableStream;
use web_sys::{
    FileSystemDirectoryHandle as FsDirHandle, FileSystemFileHandle as FsFileHandle,
    FileSystemGetDirectoryOptions as FsGetDirOptions, FileSystemGetFileOptions as FsGetFileOptions,
    FileSystemWritableFileStream as WritableFileStream,
    js_sys::{JsString, Reflect, Uint8Array},
};

use crate::{
    consts::saves::{MAIN_SAVE_FILE_NAME, SAVES_DIR, STORAGE_DIR},
    storage::{SaveList, SaveListError, SaveName, save_data::UnvalidatedSaveData},
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
pub(crate) enum StorageDirClearError {
    /// There is no window
    NoWindow,
    /// Error getting root dir
    RootDirError(DirGetterError),
    /// Error deleting storage dir
    DeleteError(JsValue),
}

impl Display for StorageDirClearError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("<StorageDirClearError as Display>::fmt() | {self:?}");
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

    /// Deletes the entire storage directory.
    pub(super) async fn clear() -> Result<(), StorageDirClearError> {
        let storage = web_sys::window()
            .ok_or(StorageDirClearError::NoWindow)?
            .navigator()
            .storage();
        let _ = storage.persist();
        let root = JsFuture::from(storage.get_directory())
            .await
            .map_err(|e| StorageDirClearError::RootDirError(DirGetterError::GetError(e)))?
            .dyn_into::<FsDirHandle>()
            .map_err(|e| StorageDirClearError::RootDirError(DirGetterError::GotWrongType(e)))?;

        JsFuture::from(root.remove_entry(STORAGE_DIR))
            .await
            .map_err(StorageDirClearError::DeleteError)?;

        Ok(())
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
        let iter = self.0.keys();

        let mut saves: Vec<SaveName> = Vec::new();
        let mut errors: Vec<SaveListError> = Vec::new();

        let str_done = JsValue::from_str("done");
        let str_value = JsValue::from_str("value");

        loop {
            // let next = iter.next().map_err(SaveListError)
            let next = match iter.next() {
                Ok(promise) => promise,
                Err(e) => {
                    errors.push(SaveListError::IterationError(e));
                    return SaveList {
                        saves: saves.into(),
                        errors: errors.into(),
                    };
                }
            };

            let next = match JsFuture::from(next).await {
                Ok(result) => result,
                Err(e) => {
                    errors.push(SaveListError::IterationError(e));
                    return SaveList {
                        saves: saves.into(),
                        errors: errors.into(),
                    };
                }
            };

            let done = match Reflect::get(&next, &str_done) {
                Ok(value) => value == JsValue::TRUE,
                Err(e) => {
                    errors.push(SaveListError::IterationError(e));
                    return SaveList {
                        saves: saves.into(),
                        errors: errors.into(),
                    };
                }
            };

            if done {
                break;
            }

            let value = match Reflect::get(&next, &str_value) {
                Ok(value) => value.dyn_into::<JsString>(),
                Err(e) => {
                    errors.push(SaveListError::NameGetError(e));
                    continue;
                }
            };

            let value = match value {
                Ok(value) => value,
                Err(e) => {
                    errors.push(SaveListError::NameTypeMismatch(e));
                    continue;
                }
            };

            saves.push(SaveName(String::from(value)));
        }

        SaveList {
            saves: saves.into(),
            errors: errors.into(),
        }
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

#[derive(Debug, Error)]
pub(crate) enum MainSaveReadError {
    /// Failed to get file
    FileGet(#[from] FileGetterError),
    /// Failed to finish streaming from file
    StreamError(JsValue),
    /// Stream yielded wrong type
    StreamWrongType(JsValue),
    /// Failed to decompress
    DecompressError(io::Error),
    /// Failed to deserialize CBOR
    DecodeError(#[from] DecodeError<Infallible>),
}

impl Display for MainSaveReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("<MainSaveReadError as Display>::fmt()");
    }
}

#[derive(Debug, Error)]
pub(crate) enum MainSaveWriteError {
    /// Failed to get file
    FileGet(#[from] FileGetterError),
    /// Failed to start writing to file
    WriteError(JsValue),
    /// Error while writing to file
    MidWriteError(JsValue),
    /// Error while committing file
    CommitError(JsValue),
}

impl Display for MainSaveWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("<MainSaveWriteError as Display>::fmt()");
    }
}

#[derive(Clone, Debug, AsRef, Deref)]
pub(super) struct MainSaveFile(#[deref] FsFileHandle);

impl MainSaveFile {
    pub(crate) async fn read(&self) -> Result<UnvalidatedSaveData, MainSaveReadError> {
        let file = JsFuture::from(self.0.get_file())
            .await
            .map_err(FileGetterError::GetError)?
            .dyn_into::<web_sys::File>()
            .map_err(FileGetterError::GotWrongType)?;

        #[expect(clippy::cast_possible_truncation)]
        #[expect(clippy::cast_sign_loss)]
        let file_size = file.size() as usize;
        let mut stream = ReadableStream::from_raw(file.stream()).into_stream();

        let mut decompressed = Vec::with_capacity(file_size);
        let mut decoder = ZlibDecoder::new(&mut decompressed);

        let mut uninit: Vec<MaybeUninit<u8>> = Vec::new();
        while let Some(result) = stream.next().await {
            let arr = result
                .map_err(MainSaveReadError::StreamError)?
                .dyn_into::<Uint8Array>()
                .map_err(MainSaveReadError::StreamWrongType)?;
            let len = arr.byte_length() as usize;

            uninit.clear();
            uninit.reserve_exact(len);

            let bytes = arr.copy_to_uninit(&mut uninit[0..len]);
            decoder
                .write_all(bytes)
                .map_err(MainSaveReadError::DecompressError)?;
        }

        drop(uninit);
        drop(decoder);

        Ok(cbor4ii::serde::from_slice::<UnvalidatedSaveData>(
            &decompressed,
        )?)
    }

    // TODO: This should take in a ValidatedSaveData after implementing saving
    /// # Unchecked Operation
    /// This function does not check for the validity of the save data bytes.
    ///
    /// # Parameters
    /// - `data`: The zlib-compressed, CBOR-encoded save data.
    pub(super) async fn write(&self, data: &[u8]) -> Result<(), MainSaveWriteError> {
        let stream = JsFuture::from(self.create_writable())
            .await
            .map_err(FileGetterError::GetError)?
            .dyn_into::<WritableFileStream>()
            .map_err(FileGetterError::GotWrongType)?;

        JsFuture::from(
            stream
                .write_with_u8_array(data)
                .map_err(MainSaveWriteError::WriteError)?,
        )
        .await
        .map_err(MainSaveWriteError::MidWriteError)?;

        JsFuture::from(stream.close())
            .await
            .map_err(MainSaveWriteError::CommitError)?;

        Ok(())
    }
}
