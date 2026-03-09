use strum::{EnumCount, VariantArray};

use crate::consts::ui::{altimeter::ALTITUDE_DIGITS, speedometer::SPEEDO_CHAR_LEN};

/// An SI prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, EnumCount, VariantArray)]
pub(crate) enum SIPrefix {
    Unit = 0,
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
}

impl SIPrefix {
    #[expect(clippy::cast_possible_truncation)]
    pub(crate) const COUNT_U8: u8 = <Self as EnumCount>::COUNT as u8;

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

    #[inline]
    #[must_use]
    pub(crate) const fn to_char(self) -> Option<char> {
        match self {
            Self::Unit => None,
            Self::Kilo => Some('k'),
            Self::Mega => Some('M'),
            Self::Giga => Some('G'),
            Self::Tera => Some('T'),
            Self::Peta => Some('P'),
            Self::Exa => Some('E'),
            Self::Zetta => Some('Z'),
            Self::Yotta => Some('Y'),
            Self::Ronna => Some('R'),
            Self::Quetta => Some('Q'),
        }
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

    /// The max speed this prefix shall be used for.
    ///
    /// This is non-inclusive; if this function returns
    /// 1e6, then a value of exactly 1e6 should not use
    /// this prefix.
    #[inline]
    #[must_use]
    pub(crate) fn max_speed(self) -> f64 {
        // 999.99  m/s
        // 1.0000 km/s

        // len = 1: 500.0
        // len = 2: 950.0
        // len = 3: 995.0 // this equation breaks at <= 3 but works fine past it
        // len = 4: 999.5
        // len = 5: 999.95
        // len = 6: 999.995
        // 1000 - 0.5 * 10^(3 - len)

        const _ASSERTION: () = assert!(SPEEDO_CHAR_LEN > 3);

        let coeff = 0.5f64.mul_add(-10.0f64.powi(3 - i32::from(SPEEDO_CHAR_LEN)), 1000.0);

        coeff * self.multiplier()
    }
}
