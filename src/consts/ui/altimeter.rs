#![cfg_attr(not(feature = "not-headless"), expect(dead_code))]

use strum::VariantArray;

use crate::consts::si::SIPrefix;

/// The amount of digits to display in the altitude number.
pub(crate) const ALTITUDE_DIGITS: u8 = 6;

/// The size of "big text" in the altimeter.
pub(crate) const ALTIMETER_BIG_TEXT_SIZE: f32 = 36.0;

/// The size of "medium text" in the altimeter.
pub(crate) const ALTIMETER_MEDIUM_TEXT_SIZE: f32 = 18.0;

/// The size of "small text" in the altimeter.
pub(crate) const ALTIMETER_SMALL_TEXT_SIZE: f32 = 14.0;

/// The size of "tiny text" in the altimeter.
pub(crate) const ALTIMETER_TINY_TEXT_SIZE: f32 = 11.0;

/// The maximum logical screen width to use the mobile
/// altimeter.
pub(crate) const ALTIMETER_MOBILE_CUTOFF: f32 = 576.0;

/// Contains data on how the altitude should be formatted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct AltitudeFormat {
    /// Is this altitude negative?
    pub(crate) is_negative: bool,
    /// The integer number to display before the SI prefix, for
    /// desktop.
    pub(crate) desktop_numeric: [char; ALTITUDE_DIGITS as usize],
    /// The number and sign to display before the SI prefix, for
    /// mobile.
    pub(crate) mobile_numeric: [char; 1 + ALTITUDE_DIGITS as usize],
    /// The SI prefix to display after the number, or "m".
    pub(crate) prefix: AltitudePrefix,
}

impl AltitudeFormat {
    pub(crate) const NAN: Self = Self {
        is_negative: true,
        desktop_numeric: ['#', 'N', 'a', 'N', '!', '#'],
        mobile_numeric: ['#', '#', 'N', 'a', 'N', '#', '#'],
        prefix: AltitudePrefix(None),
    };

    pub(crate) const OVERFLOW: Self = Self {
        is_negative: false,
        desktop_numeric: ['#', 'O', 'V', 'E', 'R', '#'],
        mobile_numeric: ['#', 'O', 'V', 'E', 'R', '!', '#'],
        prefix: AltitudePrefix(None),
    };

    pub(crate) const UNDERFLOW: Self = Self {
        is_negative: true,
        desktop_numeric: ['#', 'U', 'N', 'D', 'R', '#'],
        mobile_numeric: ['#', 'U', 'N', 'D', 'E', 'R', '#'],
        prefix: AltitudePrefix(None),
    };

    #[must_use]
    pub(crate) fn new(mut altitude: f64) -> Self {
        if altitude.is_nan() {
            return Self::NAN;
        }

        let is_negative = altitude.is_sign_negative();
        let out_of_range = if is_negative {
            Self::UNDERFLOW
        } else {
            Self::OVERFLOW
        };
        altitude = altitude.abs();

        let prefix = AltitudePrefix::from_meters(altitude);
        let mult = prefix.multiplier();
        let scaled_alt = altitude / mult;

        if scaled_alt >= 10f64.powi(i32::from(ALTITUDE_DIGITS)) {
            return out_of_range;
        }

        let desktop_numeric: [char; ALTITUDE_DIGITS as usize] = format!("{scaled_alt:06.0}")
            .chars()
            .collect::<Vec<char>>()
            .try_into()
            .unwrap_or(out_of_range.desktop_numeric);

        let mut mobile_numeric: [char; 1 + ALTITUDE_DIGITS as usize] =
            format!(" {scaled_alt:>6.0}")
                .chars()
                .collect::<Vec<char>>()
                .try_into()
                .unwrap_or(out_of_range.mobile_numeric);

        // Insert minus if needed
        if is_negative {
            let minus_position = mobile_numeric
                .into_iter()
                .position(|char| char != ' ')
                .map(|x| x - 1)
                .unwrap_or_default();

            mobile_numeric[minus_position] = '-';
        }

        Self {
            is_negative,
            desktop_numeric,
            mobile_numeric,
            prefix,
        }
    }
}

/// Contains data on the post-numeric thing to display, e.g. "k"
///
/// If the inner value is None, then this means the meter amount is out
/// of range!
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct AltitudePrefix(pub(crate) Option<SIPrefix>);

impl From<Option<SIPrefix>> for AltitudePrefix {
    fn from(value: Option<SIPrefix>) -> Self {
        Self(value)
    }
}

impl From<SIPrefix> for AltitudePrefix {
    fn from(value: SIPrefix) -> Self {
        Self(Some(value))
    }
}

impl AltitudePrefix {
    /// Get an altitude prefix to use given a nonnegative float.
    #[must_use]
    pub(crate) fn from_meters(meters: f64) -> Self {
        SIPrefix::VARIANTS
            .iter()
            .find(|prefix| prefix.max_altitude() > meters)
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
    pub(crate) const fn to_char(self) -> char {
        let Some(si) = self.0 else { return 'X' };

        match si.to_char() {
            Some(ch) => ch,
            None => 'm',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_altitude_prefix() {
        let cases = [
            (0.0, Some(SIPrefix::Unit)),
            (1.0, Some(SIPrefix::Unit)),
            (100_000.0, Some(SIPrefix::Unit)),
            (999_999.0, Some(SIPrefix::Unit)),
            (999_999.4, Some(SIPrefix::Unit)),
            (999_999.499_999_999_9, Some(SIPrefix::Unit)),
            (999_999.5, Some(SIPrefix::Kilo)),
            (999_999_000.0, Some(SIPrefix::Kilo)),
            (999_999_499.9, Some(SIPrefix::Kilo)),
            (999_999_499.999_999_9, Some(SIPrefix::Kilo)),
            (999_999_500.0, Some(SIPrefix::Mega)),
            (999_999e12, Some(SIPrefix::Tera)),
            (999_999.499e12, Some(SIPrefix::Tera)),
            (999_999.499_999_999_9e12, Some(SIPrefix::Tera)),
            (999_999.5e12, Some(SIPrefix::Peta)),
            (999_999.499_999_999_9e30, Some(SIPrefix::Quetta)),
            (999_999.5e30, None),
            (1_000_000e30, None),
            (1e38, None),
            (1e308, None),
            (f64::MAX, None),
            (f64::INFINITY, None),
            (f64::NAN, None),
        ];

        for (meters, prefix) in cases {
            assert_eq!(AltitudePrefix::from_meters(meters).0, prefix);
        }
    }

    #[test]
    fn test_format_prefix() {
        let cases = [
            0.0,
            1.0,
            100_000.0,
            999_999.0,
            999_999.4,
            999_999.499_999_999_9,
            999_999.5,
            999_999_000.0,
            999_999_499.9,
            999_999_499.999_999_9,
            999_999_500.0,
            999_999e12,
            999_999.499e12,
            999_999.499_999_999_9e12,
            999_999.5e12,
            999_999.499_999_999_9e30,
            999_999.5e30,
            1_000_000e30,
            1e38,
            1e308,
            f64::MAX,
            f64::INFINITY,
            f64::NAN,
        ];

        for meters in cases {
            let prefix = AltitudePrefix::from_meters(meters);

            assert_eq!(AltitudeFormat::new(meters).prefix, prefix);
        }
    }
}
