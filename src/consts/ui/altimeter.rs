use strum::{EnumCount, VariantArray};

/// The amount of digits to display in the altitude number.
pub(crate) const ALTITUDE_DIGITS: u8 = 6;

/// The size of "big text" in the altimeter.
pub(crate) const ALTIMETER_BIG_TEXT_SIZE: f32 = 36.0;

/// The size of "medium text" in the altimeter.
pub(crate) const ALTIMETER_MEDIUM_TEXT_SIZE: f32 = 22.0;

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
    /// The integer number to display before the SI prefix.
    pub(crate) numeric: [char; ALTITUDE_DIGITS as usize],
    /// The SI prefix to display after the number, or "m".
    pub(crate) prefix: AltitudePrefix,
}

impl AltitudeFormat {
    pub(crate) const NAN: Self = Self {
        is_negative: true,
        numeric: ['.', '.', '.', 'N', 'a', 'N'],
        prefix: AltitudePrefix::OutOfRange,
    };

    pub(crate) const OVERFLOW: Self = Self {
        is_negative: false,
        numeric: ['+', '+', '+', '+', '+', '+'],
        prefix: AltitudePrefix::OutOfRange,
    };

    pub(crate) const UNDERFLOW: Self = Self {
        is_negative: true,
        numeric: ['-', '-', '-', '-', '-', '-'],
        prefix: AltitudePrefix::OutOfRange,
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

        let Ok(chars): Result<[char; 6], _> = format!("{scaled_alt:06.0}")
            .chars()
            .collect::<Vec<char>>()
            .try_into()
        else {
            return out_of_range;
        };

        Self {
            is_negative,
            numeric: chars,
            prefix,
        }
    }
}

/// Contains data on the post-numeric thing to display, e.g. "k"
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, EnumCount, VariantArray)]
pub(crate) enum AltitudePrefix {
    Meter = 0,
    Kilo = 1,
    Mega = 2,
    Giga = 3,
    Tera = 4,
    Peta = 5,
    Exa = 6,
    Zetta = 7,
    Yotta = 8,
    Ronna = 9,
    Quetta = 10,
    OutOfRange = 11,
}

impl AltitudePrefix {
    /// Get an altitude prefix to use given a nonnegative float.
    #[must_use]
    pub(crate) fn from_meters(meters: f64) -> Self {
        Self::VARIANTS
            .iter()
            .find(|prefix| prefix.max_altitude() > meters)
            .copied()
            .unwrap_or(Self::OutOfRange)
    }

    #[inline]
    #[must_use]
    pub(crate) const fn discriminant(self) -> u8 {
        self as u8
    }

    /// The multiplier or scale of this prefix.
    #[inline]
    #[must_use]
    pub(crate) fn multiplier(self) -> f64 {
        1000f64.powi(i32::from(self.discriminant()))
    }

    /// The max altitude this prefix shall be used for.
    ///
    /// This is non-inclusive; if this function returns
    /// 1e6, then a value of exactly 1e6 should not use
    /// this prefix.
    #[inline]
    #[must_use]
    pub(crate) fn max_altitude(self) -> f64 {
        (10.0f64.powi(i32::from(ALTITUDE_DIGITS)) - 0.5) * self.multiplier()
    }

    #[must_use]
    pub(crate) const fn to_char(self) -> char {
        match self {
            Self::Meter => 'm',
            Self::Kilo => 'k',
            Self::Mega => 'M',
            Self::Giga => 'G',
            Self::Tera => 'T',
            Self::Peta => 'P',
            Self::Exa => 'E',
            Self::Zetta => 'Z',
            Self::Yotta => 'Y',
            Self::Ronna => 'R',
            Self::Quetta => 'Q',
            Self::OutOfRange => 'X',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_altitude_prefix() {
        let cases = [
            (0.0, AltitudePrefix::Meter),
            (1.0, AltitudePrefix::Meter),
            (100_000.0, AltitudePrefix::Meter),
            (999_999.0, AltitudePrefix::Meter),
            (999_999.4, AltitudePrefix::Meter),
            (999_999.499_999_999_9, AltitudePrefix::Meter),
            (999_999.5, AltitudePrefix::Kilo),
            (999_999_000.0, AltitudePrefix::Kilo),
            (999_999_499.9, AltitudePrefix::Kilo),
            (999_999_499.999_999_9, AltitudePrefix::Kilo),
            (999_999_500.0, AltitudePrefix::Mega),
            (999_999e12, AltitudePrefix::Tera),
            (999_999.499e12, AltitudePrefix::Tera),
            (999_999.499_999_999_9e12, AltitudePrefix::Tera),
            (999_999.5e12, AltitudePrefix::Peta),
            (999_999.499_999_999_9e30, AltitudePrefix::Quetta),
            (999_999.5e30, AltitudePrefix::OutOfRange),
            (1_000_000e30, AltitudePrefix::OutOfRange),
            (1e38, AltitudePrefix::OutOfRange),
            (1e308, AltitudePrefix::OutOfRange),
            (f64::MAX, AltitudePrefix::OutOfRange),
            (f64::INFINITY, AltitudePrefix::OutOfRange),
            (f64::NAN, AltitudePrefix::OutOfRange),
        ];

        for (meters, prefix) in cases {
            assert_eq!(AltitudePrefix::from_meters(meters), prefix);
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
