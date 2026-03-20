use core::fmt::Display;
use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};
use thiserror::Error;

use crate::{
    consts::saves::{DEFAULT_SAVE, SAVE_NAME_STR, nonweb::SAVE_DIR},
    fl,
    storage::{
        SaveInitError, SaveList, SaveListError as SaveListErrorWrapper, SaveName, SaveReadError,
        Storage, save_data::UnvalidatedSaveData,
    },
};

fn get_save_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join(SAVE_DIR))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NonWebStorage;

impl Storage for NonWebStorage {
    async fn init_saves(self) -> Result<(), SaveInitError> {
        let dir = get_save_dir().ok_or(SaveInitError::NoSaveDir)?;
        fs::create_dir_all(&dir).map_err(SaveInitError::DirCreation)?;

        let file = dir.join(SAVE_NAME_STR);

        // TODO: Remove this when we implement saving and multi-save-files
        fs::write(file, DEFAULT_SAVE).map_err(SaveInitError::DirCreation)
    }

    async fn get_save_list(self) -> SaveList {
        let Some(dir) = get_save_dir() else {
            return SaveList {
                saves: Box::from([]),
                errors: Box::new([SaveListErrorWrapper(SaveListError::NoSaveDir)]),
            };
        };

        let read_dir = match fs::read_dir(&dir) {
            Ok(rd) => rd,
            Err(e) => {
                return SaveList {
                    saves: Box::from([]),
                    errors: Box::from([SaveListErrorWrapper(SaveListError::ReadDirError(e))]),
                };
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

            if !file_type.is_file() {
                errors.push(SaveListError::NotAFile(path));
                continue;
            }

            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(e) => {
                    errors.push(SaveListError::MetadataFetchError { error: e, path });
                    continue;
                }
            };

            if metadata.len() == 0 {
                errors.push(SaveListError::FileEmpty(path));
                continue;
            }

            saves.push(SaveName(entry.file_name()));
        }

        SaveList {
            saves: saves.into_boxed_slice(),
            errors: errors.into_iter().map(SaveListErrorWrapper).collect(),
        }
    }

    async fn load(self, save_name: &SaveName) -> Result<UnvalidatedSaveData, SaveReadError> {
        let dir = get_save_dir().ok_or(SaveReadError::NoSaveDir)?;

        let savefile_path = dir.join(&save_name.0);
        let mut savefile = fs::File::open(savefile_path)?;

        let mut save_str = String::with_capacity(
            savefile
                .metadata()
                .map(|m| m.len())
                .unwrap_or_default()
                .try_into()
                .unwrap_or_default(),
        );

        savefile.read_to_string(&mut save_str)?;

        Ok(serde_json::from_str(&save_str)?)
    }
}

#[derive(Debug, Error)]
pub(super) enum SaveListError {
    /// Couldn't decide on a save dir
    NoSaveDir,
    /// Couldn't read the save dir
    ReadDirError(io::Error),
    /// Couldn't read a save dir entry
    DirEntryError(io::Error),
    /// Couldn't read an entry's file type
    FileTypeError { path: PathBuf, error: io::Error },
    /// Dir entry isn't a file
    NotAFile(PathBuf),
    /// Couldn't fetch file metadata
    MetadataFetchError { path: PathBuf, error: io::Error },
    /// Save file is empty
    FileEmpty(PathBuf),
}

impl Display for SaveListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSaveDir => f.write_str(&fl!("error__saveGeneral__noSaveDir")),
            Self::ReadDirError(inner) => {
                f.write_str(&fl!("error__saveList__readDir", inner = inner.to_string()))
            }
            Self::DirEntryError(inner) => {
                f.write_str(&fl!("error__saveList__dirEntry", inner = inner.to_string()))
            }
            Self::FileTypeError { path, error } => f.write_str(&fl!(
                "error__saveList__fileType",
                path = path.to_string_lossy(),
                inner = error.to_string()
            )),
            Self::NotAFile(path) => f.write_str(&fl!(
                "error__saveList__notFile",
                path = path.to_string_lossy()
            )),
            Self::MetadataFetchError { path, error } => f.write_str(&fl!(
                "error__saveList__metadataFetch",
                path = path.to_string_lossy(),
                inner = error.to_string()
            )),
            Self::FileEmpty(path) => f.write_str(&fl!(
                "error__saveList__fileEmpty",
                path = path.to_string_lossy()
            )),
        }
    }
}
