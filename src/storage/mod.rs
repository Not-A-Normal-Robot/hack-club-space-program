#[cfg(not(target_family = "wasm"))]
use bevy::tasks::futures_lite::io;
use bevy::{log::warn, platform::collections::HashSet};
use core::fmt::{Debug, Display};
#[cfg(not(target_family = "wasm"))]
use std::ffi::OsString;
use std::{borrow::Cow, sync::Mutex};
use thiserror::Error;

use crate::{
    fl,
    storage::save_data::{SaveDataError, UnvalidatedSaveData},
};

pub(crate) mod save_data;

#[cfg(not(target_family = "wasm"))]
mod nonweb;
#[cfg(target_family = "wasm")]
mod web;

pub(crate) trait Storage: Copy + Sized + Send + Sync {
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

    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    // async fn get_save_list(&self) -> SaveList;
    #[cfg(not(target_family = "wasm"))]
    fn get_save_list(self) -> impl Future<Output = SaveList> + Send;

    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    // async fn get_save_list(&self) -> SaveList;
    #[cfg(target_family = "wasm")]
    async fn get_save_list(self) -> SaveList;

    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    #[cfg(not(target_family = "wasm"))]
    fn load(
        self,
        save_name: &SaveName,
    ) -> impl Future<Output = Result<UnvalidatedSaveData, SaveReadError>> + Send;

    /// # Blocking
    /// This function may block.
    /// Please run this in an [`IoTaskPool`][bevy::tasks::IoTaskPool]
    #[cfg(target_family = "wasm")]
    async fn load(self, save_name: &SaveName) -> Result<UnvalidatedSaveData, SaveReadError>;
}

pub(crate) fn get_storage() -> impl Storage {
    #[cfg(target_family = "wasm")]
    {
        return web::WebStorage;
    }

    #[cfg(not(target_family = "wasm"))]
    {
        nonweb::NonWebStorage
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
                "error__saveInit__factoryInit",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::DbOpenRequest(inner) => f.write_str(&fl!(
                "error__saveInit__dbOpenRequest",
                inner = inner.to_string()
            )),
            #[cfg(target_family = "wasm")]
            Self::DbOpen(inner) => {
                f.write_str(&fl!("error__saveInit__dbOpen", inner = inner.to_string()))
            }
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
            return Self(value);
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
            return Cow::Borrowed(&self.0);
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

#[derive(Debug)]
pub(crate) struct SaveList {
    pub(crate) saves: Box<[SaveName]>,
    pub(crate) errors: Box<[SaveListError]>,
}

#[derive(Debug, Error)]
pub(crate) struct SaveListErrors(pub(crate) Box<[SaveListError]>);

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

#[derive(Error)]
#[repr(transparent)]
pub(crate) struct SaveListError(SaveListErrorInner);

#[cfg(not(target_family = "wasm"))]
type SaveListErrorInner = nonweb::SaveListError;

#[cfg(target_family = "wasm")]
type SaveListErrorInner = web::SaveListError;

impl Debug for SaveListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <SaveListErrorInner as Debug>::fmt(&self.0, f)
    }
}

impl Display for SaveListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // self.0.fmt(f)
        <SaveListErrorInner as Display>::fmt(&self.0, f)
    }
}

#[derive(Debug, Error)]
pub(crate) enum SaveReadError {
    #[cfg(not(target_family = "wasm"))]
    NoSaveDir,
    #[cfg(not(target_family = "wasm"))]
    IoError(#[from] io::Error),
    ParseError(#[from] serde_json::Error),
    InvalidState(#[from] SaveDataError),
}

impl Display for SaveReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(not(target_family = "wasm"))]
            Self::NoSaveDir => f.write_str(&fl!("error__saveGeneral__noSaveDir")),
            #[cfg(not(target_family = "wasm"))]
            Self::IoError(e) => {
                f.write_str(&fl!("error__saveRead__ioError", inner = format!("{e}")))
            }
            Self::InvalidState(e) => <SaveDataError as Display>::fmt(e, f),
            Self::ParseError(error) => f.write_str(&fl!(
                "error__saveRead__parseError",
                inner = format!("{error}")
            )),
        }
    }
}
