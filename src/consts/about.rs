use crate::{
    assets::fonts::{LICENSE_DOTO, LICENSE_JETBRAINS_MONO, LICENSE_WDXL},
    components::about_menu::{
        AsideElement, BackButton, HeaderTitle, MainAsideSeparator, MainElement, RootNode, TabText,
    },
    fl,
};
use bevy::prelude::*;
use std::borrow::Cow;

pub(crate) const ABOUT_ENTRY_COUNT: usize = 5;

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
        4 => fl!("aboutMenu__article__jbmLicense__title"),
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
        4 => LICENSE_JETBRAINS_MONO.into(),
        ABOUT_ENTRY_COUNT.. => unreachable!("index {index} should not be >= {ABOUT_ENTRY_COUNT}"),
    }
}

pub(crate) type ResponsiveQuery<'w, 's, 'qw, 'qs> = ParamSet<
    'w,
    's,
    (
        Query<'qw, 'qs, &'static mut Node, With<RootNode>>,
        Query<'qw, 'qs, &'static mut Node, With<HeaderTitle>>,
        Query<'qw, 'qs, &'static mut Node, With<BackButton>>,
        Query<'qw, 'qs, &'static mut Node, With<MainElement>>,
        Query<'qw, 'qs, &'static mut Node, With<AsideElement>>,
        Query<'qw, 'qs, &'static mut Node, With<MainAsideSeparator>>,
        Query<'qw, 'qs, &'static mut TextLayout, With<TabText>>,
    ),
>;

pub(crate) const ASIDE_FONT_SIZE: f32 = 24.0;
pub(crate) const MAIN_FONT_SIZE: f32 = 21.0;
