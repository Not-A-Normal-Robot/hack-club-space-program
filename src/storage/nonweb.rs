use core::fmt::Display;
use std::{
    ffi::OsStr,
    fs,
    io::{self, Read},
    path::PathBuf,
};

use thiserror::Error;

use crate::{
    fl,
    storage::{
        SaveList, SaveListError as SaveListErrorWrapper, SaveName, SaveReadError,
        save_data::UnvalidatedSaveData,
    },
};

fn get_save_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("hack-club-space-program/saves"))
}

#[derive(Debug, Error)]
pub(super) enum SaveListError {
    /// Couldn't decide on a save dir
    NoSaveDir,
    /// Couldn't create the save dir
    DirCreationError(io::Error),
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
    EmptyFile(PathBuf),
}

impl Display for SaveListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSaveDir => f.write_str(&fl!("error__saveGeneral__noSaveDir")),
            _ => todo!("Display impl for other `SaveListError`s"),
        }
    }
}

// TODO: Make this async
#[expect(dead_code)]
pub(super) async fn get_save_list() -> SaveList {
    let Some(dir) = get_save_dir() else {
        return SaveList {
            saves: Box::from([]),
            errors: Box::new([SaveListErrorWrapper(SaveListError::NoSaveDir)]),
        };
    };

    if let Err(e) = fs::create_dir_all(&dir) {
        return SaveList {
            saves: Box::from([]),
            errors: Box::from([SaveListErrorWrapper(SaveListError::DirCreationError(e))]),
        };
    }

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
            errors.push(SaveListError::EmptyFile(path));
            continue;
        }

        saves.push(SaveName(entry.file_name()));
    }

    SaveList {
        saves: saves.into_boxed_slice(),
        errors: errors
            .into_iter()
            .map(|e| SaveListErrorWrapper(e))
            .collect(),
    }
}

pub(super) async fn load(save_name: &OsStr) -> Result<UnvalidatedSaveData, SaveReadError> {
    let dir = get_save_dir().ok_or(SaveReadError::NoSaveDir)?;
    let savefile = dir.join(save_name);

    let savefile_path = dir.join(save_name);
    let mut savefile = fs::File::open(savefile_path)?;
    savefile.lock();

    // TODO: Make this async
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
