use bevy::prelude::*;

use crate::assets::{fonts::initialize_fonts, icons::initialize_icons};

pub(crate) mod fonts;
pub(crate) mod icons;

pub(crate) fn initialize_assets(app: &mut App) {
    initialize_fonts(app);
    initialize_icons(app);
}
