use std::borrow::Cow;

use crate::{
    assets::fonts::{LICENSE_DOTO, LICENSE_WDXL},
    fl,
};

pub(crate) const ABOUT_ENTRY_COUNT: usize = 4;

/// Get the list of article titles for the current locale.
///
/// # Panics
/// Panics if `index >= ABOUT_ENTRY_COUNT`.
pub(crate) fn load_article_title(index: usize) -> String {
    match index {
        0 => fl!("aboutMenu__article__main__title"),
        1 => fl!("aboutMenu__article__gameLicense__title"),
        2 => fl!("aboutMenu__article__dotoLicense__title"),
        3 => fl!("aboutMenu__article__wdxlLicense__title"),
        ABOUT_ENTRY_COUNT.. => unreachable!("index {index} should not be >= {ABOUT_ENTRY_COUNT}"),
    }
}

/// Get the article at the specified index for the current locale.
///
/// # Panics
/// Panics if `index >= ABOUT_ENTRY_COUNT`.
pub(crate) fn load_article(index: usize) -> Cow<'static, str> {
    match index {
        0 => fl!("aboutMenu__article__main__body").into(),
        1 => include_str!("../../LICENSE").into(),
        2 => LICENSE_DOTO.into(),
        3 => LICENSE_WDXL.into(),
        ABOUT_ENTRY_COUNT.. => unreachable!("index {index} should not be >= {ABOUT_ENTRY_COUNT}"),
    }
}
