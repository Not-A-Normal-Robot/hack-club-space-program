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
    pub(crate) hspd: [char; SPEEDO_CHAR_LEN as usize],
    /// The formatted vertical speed.
    pub(crate) vspd: [char; SPEEDO_CHAR_LEN as usize],
    /// The formatted total speed.
    pub(crate) tspd: [char; SPEEDO_CHAR_LEN as usize],
    /// The speedometer unit to use.
    pub(crate) unit: SpeedometerUnit,
}

impl SpeedometerFormat {
    const TEXT_OVERFLOW: [char; SPEEDO_CHAR_LEN as usize] = ['>', '9', '9', '9', '.', '9'];
    const TEXT_UNDERFLOW: [char; SPEEDO_CHAR_LEN as usize] = ['<', '-', '1', '0', '0', '0'];
    const TEXT_NAN: [char; SPEEDO_CHAR_LEN as usize] = ['#', 'N', 'a', 'N', '!', '#'];

    /// Gets the formatted speedometer values from the given
    /// horizontal, vertical, and total speed values, in m/s.
    pub(crate) fn from_speeds(hspd: f64, vspd: f64, tspd: f64) -> Self {
        debug_assert!(tspd >= hspd);
        debug_assert!(tspd >= vspd);

        let unit = SpeedometerUnit::from_speed(tspd);
        let mult = unit.multiplier();

        let [hspd, vspd, tspd] = [hspd, vspd, tspd]
            .map(|x| x / mult)
            .map(Self::format_scaled);

        Self {
            hspd,
            vspd,
            tspd,
            unit,
        }
    }

    /// Formats a scaled number into a char array.
    fn format_scaled(scaled: f64) -> [char; SPEEDO_CHAR_LEN as usize] {
        if scaled.is_nan() {
            return Self::TEXT_NAN;
        } else if scaled >= SIPrefix::Unit.max_speed(SPEEDO_CHAR_LEN) {
            return Self::TEXT_OVERFLOW;
        } else if -scaled >= SIPrefix::Unit.max_speed(SPEEDO_CHAR_LEN - 1) {
            return Self::TEXT_UNDERFLOW;
        }

        let precision = Self::get_precision(scaled);

        format!("{scaled:.*}", precision as usize)
            .chars()
            .collect::<Vec<_>>()
            .try_into()
            .expect("resulting string should fit within char length bounds")
    }

    /// Gets how many digits of precision to display.
    fn get_precision(scaled: f64) -> u8 {
        // Cutoffs:
        // 9.99995
        // 99.9995
        // 999.995

        // 10^1 - 0.5 * 10^-4
        // 10^2 - 0.5 * 10^-3
        // 10^3 - 0.5 * 10^-2

        let is_negative = scaled.is_sign_negative();
        let scaled = scaled.abs();
        let chars = if is_negative {
            SPEEDO_CHAR_LEN - 1
        } else {
            SPEEDO_CHAR_LEN
        };

        for precision in (1..chars - 1).rev() {
            let max = 0.5f64.mul_add(
                -10f64.powi(-i32::from(precision)),
                10f64.powi(i32::from(chars) - i32::from(precision) - 1),
            );

            if scaled < max {
                return precision;
            }
        }

        0
    }
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
        let chars = if speed.is_sign_negative() {
            SPEEDO_CHAR_LEN - 1
        } else {
            SPEEDO_CHAR_LEN
        };

        SIPrefix::VARIANTS
            .iter()
            .find(|prefix| prefix.max_speed(chars) > speed)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speedometer_unit() {
        let cases = [
            (0.0000, SpeedometerUnit::from(SIPrefix::Unit)),
            (1.0000, SpeedometerUnit::from(SIPrefix::Unit)),
            (999.994_999_999_999_9, SpeedometerUnit::from(SIPrefix::Unit)),
            (999.995, SpeedometerUnit::from(SIPrefix::Kilo)),
            (
                999.994_999_999_999_9e3,
                SpeedometerUnit::from(SIPrefix::Kilo),
            ),
            (999.995e3, SpeedometerUnit::from(SIPrefix::Mega)),
            (
                999.994_999_999_999_9e30,
                SpeedometerUnit::from(SIPrefix::Quetta),
            ),
            (999.995e30, SpeedometerUnit::from(None::<SIPrefix>)),
            (f64::NAN, SpeedometerUnit::from(None::<SIPrefix>)),
            (f64::INFINITY, SpeedometerUnit::from(None::<SIPrefix>)),
        ];

        for (input, expected) in cases {
            let unit = SpeedometerUnit::from_speed(input);
            assert_eq!(unit, expected);
        }
    }

    #[test]
    fn test_get_precision() {
        let cases = [
            (-999.95, 0),
            (-999.949_999_999_999_9, 1),
            (-99.995, 1),
            (-99.994_999_999_999_99, 2),
            (-9.999_5, 2),
            (-9.999_499_999_999_998, 3),
            (-9.999, 3),
            (-1.000, 3),
            (-0.4039, 3),
            (-0.000, 3),
            (0.0000, 4),
            (1.0000, 4),
            (9.9999, 4),
            (9.999_949_999_999_999, 4),
            (9.99995, 3),
            (10.000, 3),
            (99.999, 3),
            (99.999_499_999_999_99, 3),
            (99.9995, 2),
            (100.00, 2),
            (999.99, 2),
            (999.994_999_999_999_9, 2),
            (999.995, 1),
            (1000.0, 1),
            (9999.9, 1),
            (9_999.949_999_999_999, 1),
            (9999.95, 0),
            (10000., 0),
        ];

        for (input, expected) in cases {
            let gotten = SpeedometerFormat::get_precision(input);
            assert_eq!(gotten, expected, "assertion failed with input {input}");
        }
    }

    #[test]
    fn test_format_scaled() {
        let cases = [
            (-1000., SpeedometerFormat::TEXT_UNDERFLOW),
            (-999.95, SpeedometerFormat::TEXT_UNDERFLOW),
            (-999.949_999_999_999_9, ['-', '9', '9', '9', '.', '9']),
            (-400.39, ['-', '4', '0', '0', '.', '4']),
            (-100.0, ['-', '1', '0', '0', '.', '0']),
            (-99.995, ['-', '1', '0', '0', '.', '0']),
            (-99.994_999_999_999_99, ['-', '9', '9', '.', '9', '9']),
            (-40.039, ['-', '4', '0', '.', '0', '4']),
            (-10.0, ['-', '1', '0', '.', '0', '0']),
            (-9.999_5, ['-', '1', '0', '.', '0', '0']),
            (-9.999_499_999_999_998, ['-', '9', '.', '9', '9', '9']),
            (-9.999, ['-', '9', '.', '9', '9', '9']),
            (-4.0039, ['-', '4', '.', '0', '0', '4']),
            (-0.4039, ['-', '0', '.', '4', '0', '4']),
            (-0.000_00, ['-', '0', '.', '0', '0', '0']),
            (0.000_00, ['0', '.', '0', '0', '0', '0']),
            (0.000_04, ['0', '.', '0', '0', '0', '0']),
            (
                0.000_049_999_999_999_999_999,
                ['0', '.', '0', '0', '0', '0'],
            ),
            (0.000_05, ['0', '.', '0', '0', '0', '1']),
            (0.0001, ['0', '.', '0', '0', '0', '1']),
            (0.00016, ['0', '.', '0', '0', '0', '2']),
            (0.40016, ['0', '.', '4', '0', '0', '2']),
            (1.0000, ['1', '.', '0', '0', '0', '0']),
            (4.00036, ['4', '.', '0', '0', '0', '4']),
            (9.0000, ['9', '.', '0', '0', '0', '0']),
            (9.9999, ['9', '.', '9', '9', '9', '9']),
            (9.999_94, ['9', '.', '9', '9', '9', '9']),
            (9.999_949_999_999_999, ['9', '.', '9', '9', '9', '9']),
            (9.999_95, ['1', '0', '.', '0', '0', '0']),
            (40.0036, ['4', '0', '.', '0', '0', '4']),
            (99.999_4, ['9', '9', '.', '9', '9', '9']),
            (99.999_499_999_999_99, ['9', '9', '.', '9', '9', '9']),
            (99.999_5, ['1', '0', '0', '.', '0', '0']),
            (400.036, ['4', '0', '0', '.', '0', '4']),
            (999.994, ['9', '9', '9', '.', '9', '9']),
            (999.994_999_999_999_9, ['9', '9', '9', '.', '9', '9']),
            (999.995, SpeedometerFormat::TEXT_OVERFLOW),
            (1000.0, SpeedometerFormat::TEXT_OVERFLOW),
            (1e38, SpeedometerFormat::TEXT_OVERFLOW),
            (1e308, SpeedometerFormat::TEXT_OVERFLOW),
            (f64::INFINITY, SpeedometerFormat::TEXT_OVERFLOW),
            (f64::NAN, SpeedometerFormat::TEXT_NAN),
        ];

        for (input, expected) in cases {
            let gotten = SpeedometerFormat::format_scaled(input);
            assert_eq!(gotten, expected, "assertion failed with input {input}");
        }
    }
}
