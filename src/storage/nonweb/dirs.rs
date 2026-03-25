//! Abstractions of the game's storage directories.
//!
//! Starting point: [`StorageDir`]

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use derive_more::AsRef;

use crate::{
    consts::saves::nonweb::{MAIN_SAVE_FILE_NAME, SAVES_DIR, STORAGE_DIR},
    storage::{SaveList, SaveListError, SaveName},
};

/// An abstraction of a directory for the game's
/// storage subsystem.
pub(crate) trait DirAbstraction {
    /// Gets the path of this directory.
    fn get_path(&self) -> &Path;
    /// Recursively ensures that this corresponding path exists.
    fn ensure(&self) -> io::Result<()>;
    /// Checks whether or not this path exists.
    fn exists(&self) -> io::Result<bool>;
}

impl<T> DirAbstraction for T
where
    T: AsRef<Path>,
{
    fn get_path(&self) -> &Path {
        self.as_ref()
    }

    fn ensure(&self) -> io::Result<()> {
        fs::create_dir_all(self.as_ref())
    }

    fn exists(&self) -> io::Result<bool> {
        fs::exists(&self.as_ref())
    }
}

/// An abstraction of the game's storage directory.
///
/// Path: `<STORAGE>`
#[derive(AsRef)]
#[as_ref(forward)]
pub(crate) struct StorageDir(PathBuf);

impl StorageDir {
    /// Returns [`None`] if `dirs::data_dir()` returns [`None`].
    ///
    /// # Unchecked Operation
    /// This function does not check for the existence of the
    /// corresponding path. To ensure its existence, call
    /// [`StorageDir::ensure()`].
    pub(crate) fn new() -> Option<Self> {
        if cfg!(test) {
            Some(Self(PathBuf::from("./target/testing-save-dir")))
        } else {
            dirs::data_dir().map(|dir| dir.join(STORAGE_DIR)).map(Self)
        }
    }

    /// Get an abstraction to the game's saves directory.
    ///
    /// # Unchecked Operation
    /// This function does not check for the existence of the
    /// corresponding path. To ensure its existence, call
    /// [`SavesDir::ensure()`].
    pub(crate) fn saves(&self) -> SavesDir {
        SavesDir(self.get_path().join(SAVES_DIR))
    }
}

/// An abstraction of the game's saves directory.
///
/// Path: `<STORAGE>/saves`
///
/// Get this struct using [`StorageDir::saves()`].
#[derive(AsRef)]
#[as_ref(forward)]
pub(crate) struct SavesDir(PathBuf);

impl SavesDir {
    pub(crate) fn list_saves(&self) -> SaveList {
        let read_dir = match fs::read_dir(&self.0) {
            Ok(rd) => rd,
            Err(e) => {
                return SaveListError::ReadDirError(e).into();
            }
        };

        let mut saves: Vec<SaveName> =
            Vec::with_capacity(read_dir.size_hint().1.unwrap_or(read_dir.size_hint().0));
        let mut errors: Vec<SaveListError> = Vec::new();

        for entry in read_dir {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    errors.push(SaveListError::DirEntryError(e));
                    continue;
                }
            };

            let path = entry.path();

            let file_type = match entry.file_type() {
                Ok(ty) => ty,
                Err(e) => {
                    errors.push(SaveListError::FileTypeError { path, error: e });
                    continue;
                }
            };

            if !file_type.is_dir() {
                errors.push(SaveListError::NotADir(path));
                continue;
            }

            saves.push(SaveName(entry.file_name()));
        }

        SaveList {
            saves: saves.into(),
            errors: errors.into(),
        }
    }

    /// Gets a specific save's directory.
    ///
    /// # Unchecked Operation
    /// This function does not check for the existence of
    /// the save directory. To check if it exists,
    /// call [`SaveDir::exists()`].
    pub(crate) fn get_save(&self, save_name: &SaveName) -> SaveDir {
        SaveDir(self.get_path().join(&save_name.0))
    }
}

/// An abstraction of a specific save's directory.
///
/// Path: `<STORAGE>/saves/<SAVE>`
///
/// Get this struct using [`SavesDir::get_save()`].
#[derive(AsRef)]
#[as_ref(forward)]
pub(crate) struct SaveDir(PathBuf);

impl SaveDir {
    /// Gets the main save file of this save.
    pub(crate) fn main_save(&self) -> PathBuf {
        self.0.join(MAIN_SAVE_FILE_NAME)
    }
}
