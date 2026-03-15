use core::fmt::{Debug, Display};
use std::{borrow::Cow, ffi::OsString, sync::Mutex};

#[cfg(not(target_family = "wasm"))]
use bevy::tasks::futures_lite::io;
use bevy::{log::warn, platform::collections::HashSet};
use thiserror::Error;

use crate::{
    consts::saves::DEFAULT_SAVE,
    fl,
    storage::save_data::{SaveDataError, ValidatedSaveData},
};

#[cfg(not(target_family = "wasm"))]
pub(crate) mod nonweb;
pub(crate) mod save_data;
#[cfg(target_family = "wasm")]
pub(crate) mod web;

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

static UI_CALLER_SET: Mutex<HashSet<String>> = Mutex::new(HashSet::new());

impl SaveListErrors {
    #[cold]
    fn warn_ui_once(caller: &core::panic::Location) {
        let called = UI_CALLER_SET.try_lock().map_or(false, |mut set| {
            let file = caller.file();
            if set.contains(file) {
                return true;
            }

            !set.insert(file.to_owned())
        });

        if !called {
            warn!(
                "<{} as Display>::fmt called from {caller};
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
pub(crate) struct SaveListError(
    #[cfg(not(target_family = "wasm"))] nonweb::SaveListError,
    #[cfg(target_family = "wasm")] web::SaveListError,
);

impl Debug for SaveListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl Display for SaveListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // self.0.fmt(f)
        <Self as Display>::fmt(self, f)
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
                f.write_str(&fl!("error__saveGeneral__ioError", inner = format!("{e}")))
            }
            Self::InvalidState(e) => <SaveDataError as Display>::fmt(e, f),
            Self::ParseError(error) => todo!(),
        }
    }
}

#[expect(clippy::unused_async, reason = "This function will be async soon")]
pub(crate) async fn get_save_list() -> Result<SaveList, SaveListError> {
    // TODO: Actually fetch save list
    Ok(SaveList {
        saves: Box::from([SaveName(DEFAULT_SAVE.into())]),
        errors: Box::from([]),
    })
}

pub(crate) async fn load(save_name: &SaveName) -> Result<ValidatedSaveData, SaveReadError> {
    #[cfg(target_family = "wasm")]
    let unvalidated = web::load(&save_name.0).await;
    #[cfg(not(target_family = "wasm"))]
    let unvalidated = nonweb::load(&save_name.0).await;

    Ok(unvalidated?.validate()?)
}
