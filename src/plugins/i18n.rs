use crate::consts::FLUENT_LANGUAGE_LOADER;
use bevy::prelude::*;
use i18n_embed::LanguageLoader;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "locales/"]
struct Localizations;

pub(in crate::plugins) struct I18nPlugin;

impl Plugin for I18nPlugin {
    fn build(&self, _app: &mut App) {
        load_localizations();
    }
}

pub(crate) fn load_localizations() {
    FLUENT_LANGUAGE_LOADER
        .load_languages(
            &Localizations,
            &[FLUENT_LANGUAGE_LOADER.fallback_language().clone()],
        )
        .expect("Error loading languages");
}
