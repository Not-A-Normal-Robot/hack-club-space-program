use bevy::prelude::*;

#[cfg(feature = "not-headless")]
pub(crate) mod fonts;
#[cfg(feature = "not-headless")]
pub(crate) mod icons;

pub(crate) fn initialize_assets(app: &mut App) {
    #[cfg(feature = "not-headless")]
    fonts::initialize_fonts(app);
    #[cfg(feature = "not-headless")]
    icons::initialize_icons(app);
}
