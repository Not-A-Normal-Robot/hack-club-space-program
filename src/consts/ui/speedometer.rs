use core::fmt::{Display, Write};

use strum::VariantArray;

use crate::consts::si::SIPrefix;

/// Font size used for the directional speed displays,
/// i.e., HSPD and VSPD.
pub(crate) const DIRECTIONAL_FONT_SIZE: f32 = 16.0;
/// Font size used for the number of the total speed
/// counter.
pub(crate) const TSPD_FONT_SIZE: f32 = 28.0;
/// Font size used for the speed unit label, e.g., m/s
pub(crate) const UNIT_FONT_SIZE: f32 = 18.0;

/// How many characters the speedometer speed displays
/// should use.
pub(crate) const SPEEDO_CHAR_LEN: u8 = 6;

pub(crate) struct SpeedometerFormat {
    /// The formatted horizontal speed.
    hspd: [char; SPEEDO_CHAR_LEN as usize],
    /// The formatted vertical speed.
    vspd: [char; SPEEDO_CHAR_LEN as usize],
    /// The formatted total speed.
    tspd: [char; SPEEDO_CHAR_LEN as usize],
    /// The speedometer unit to use.
    unit: SpeedometerUnit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SpeedometerUnit(pub(crate) Option<SIPrefix>);

impl From<Option<SIPrefix>> for SpeedometerUnit {
    fn from(value: Option<SIPrefix>) -> Self {
        Self(value)
    }
}

impl From<SIPrefix> for SpeedometerUnit {
    fn from(value: SIPrefix) -> Self {
        Self(Some(value))
    }
}

impl Display for SpeedometerUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_text()
            .into_iter()
            .try_for_each(|ch| f.write_char(ch))
    }
}

impl SpeedometerUnit {
    pub(crate) const CHAR_LEN: u8 = 4;
    pub(crate) const CHARS_OUT_OF_RANGE: [char; Self::CHAR_LEN as usize] = ['?', '!', '?', '!'];
    pub(crate) const CHARS_SUFFIX: [char; Self::CHAR_LEN as usize - 1] = ['m', '/', 's'];

    /// Gets the speedometer unit from the given speed in m/s.
    #[inline]
    #[must_use]
    pub(crate) fn from_speed(speed: f64) -> Self {
        SIPrefix::VARIANTS
            .iter()
            .find(|prefix| prefix.max_speed() > speed)
            .copied()
            .into()
    }

    #[must_use]
    pub(crate) fn multiplier(self) -> f64 {
        1000f64.powi(i32::from(
            self.0.map_or(SIPrefix::COUNT_U8, SIPrefix::discriminant),
        ))
    }

    #[inline]
    #[must_use]
    pub(crate) const fn to_text(self) -> [char; Self::CHAR_LEN as usize] {
        let Some(si) = self.0 else {
            return Self::CHARS_OUT_OF_RANGE;
        };

        let first = match si.to_char() {
            Some(ch) => ch,
            None => ' ',
        };

        [
            first,
            Self::CHARS_SUFFIX[0],
            Self::CHARS_SUFFIX[1],
            Self::CHARS_SUFFIX[2],
        ]
    }
}
