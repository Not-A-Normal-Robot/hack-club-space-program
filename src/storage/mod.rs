use core::fmt::Display;
use std::{borrow::Cow, ffi::OsString};

use thiserror::Error;

use crate::{
    consts::saves::SAVE_NAME,
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

#[derive(Clone, Debug, PartialEq, Error)]
pub(crate) enum SaveReadError {
    InvalidState(#[from] SaveDataError),
}

impl Display for SaveReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidState(e) => e.fmt(f),
        }
    }
}

pub(crate) fn get_save_list() -> Box<[SaveName]> {
    Box::from([SAVE_NAME.clone()])
}

pub(crate) fn load(save_name: &SaveName) -> Result<ValidatedSaveData, SaveReadError> {
    #[cfg(target_family = "wasm")]
    let unvalidated = web::load(&save_name.0);
    #[cfg(not(target_family = "wasm"))]
    let unvalidated = nonweb::load(&save_name.0);

    Ok(unvalidated?.validate()?)
}
