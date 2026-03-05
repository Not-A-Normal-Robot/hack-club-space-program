//! The color scheme for this app.
//!
//! From: <https://material-foundation.github.io/material-theme-builder/?primary=%23B39A00&colorMatch=false>

use bevy::prelude::*;

/// Converts a hex ASCII char into a nibble.
///
/// # Panics
/// This function panics if the ASCII char pair isn't valid hex.
const fn hex_to_nibble(hex: u8) -> u8 {
    let hex = hex.to_ascii_lowercase();
    if hex.is_ascii_digit() {
        return hex - b'0';
    }

    assert!(hex.is_ascii_hexdigit());

    hex - b'a' + 10
}

/// Converts hex ASCII char pair into a u8 channel.
///
/// # Panics
/// This function panics if the ASCII char pair isn't valid hex.
const fn hex_to_channel(hex: [u8; 2]) -> u8 {
    let lower = hex_to_nibble(hex[1]);
    let higher = hex_to_nibble(hex[0]);
    (higher << 4) + lower
}

/// Converts hex ASCII char pair into an f32 channel.
///
/// # Panics
/// This function panics if the ASCII char pair isn't valid hex.
const fn hex_to_float(hex: [u8; 2]) -> f32 {
    hex_to_channel(hex) as f32 / 255.0
}

/// Converts ASCII hex bytes in the form `#RRGGBB` to SRGB color space.
///
/// # Panics
/// This function panics if the ASCII hex bytes isn't valid hex.
#[expect(clippy::trivially_copy_pass_by_ref)]
const fn hex_to_color(hex: &[u8; 7]) -> Color {
    let r = hex_to_float([hex[1], hex[2]]);
    let g = hex_to_float([hex[3], hex[4]]);
    let b = hex_to_float([hex[5], hex[6]]);

    Color::Srgba(Srgba::rgb(r, g, b))
}

pub(crate) const CONTROL_MODE_BACKGROUND: Color =
    Color::Srgba(Srgba::new(0.085, 0.125, 0.105, 0.9));
pub(crate) const CONTROL_MODE_FOREGROUND: Color = hex_to_color(b"#39BC22");
pub(crate) const ORIBAR_BACKGROUND: Color = Color::Srgba(Srgba::new(0.25, 0.25, 0.25, 0.55));
pub(crate) const ALTIMETER_BACKGROUND: Color = scheme::ON_PRIMARY;
pub(crate) const ALTIMETER_ACTIVE: Color = scheme::PRIMARY;
pub(crate) const ALTIMETER_INACTIVE: Color = scheme::PRIMARY_CONTAINER;
pub(crate) const ALTIMETER_PREFIX: Color = scheme::ON_PRIMARY_CONTAINER;
pub(crate) const ALTIMETER_OUTER_BORDER: Color = shades::PRIMARY_50;
pub(crate) const ALTIMETER_INNER_BORDER: Color = shades::PRIMARY_35;

#[allow(dead_code)]
pub(crate) mod icons {
    use crate::consts::colors::hex_to_color;
    use bevy::color::Color;

    pub(crate) const COLOR_PROGRADE: Color = hex_to_color(b"#9ACD32");
}

#[allow(dead_code)]
pub(crate) mod scheme {
    use crate::consts::colors::hex_to_color;
    use bevy::color::Color;

    pub(crate) const PRIMARY: Color = hex_to_color(b"#DAC66F");
    pub(crate) const SURFACE_TINT: Color = hex_to_color(b"#DAC66F");
    pub(crate) const ON_PRIMARY: Color = hex_to_color(b"#393000");
    pub(crate) const PRIMARY_CONTAINER: Color = hex_to_color(b"#534600");
    pub(crate) const ON_PRIMARY_CONTAINER: Color = hex_to_color(b"#F8E288");
    pub(crate) const SECONDARY: Color = hex_to_color(b"#D1C6A2");
    pub(crate) const ON_SECONDARY: Color = hex_to_color(b"#363016");
    pub(crate) const SECONDARY_CONTAINER: Color = hex_to_color(b"#4D472A");
    pub(crate) const ON_SECONDARY_CONTAINER: Color = hex_to_color(b"#EEE2BC");
    pub(crate) const TERTIARY: Color = hex_to_color(b"#A9D0B3");
    pub(crate) const ON_TERTIARY: Color = hex_to_color(b"#143723");
    pub(crate) const TERTIARY_CONTAINER: Color = hex_to_color(b"#2B4E38");
    pub(crate) const ON_TERTIARY_CONTAINER: Color = hex_to_color(b"#C4ECCF");
    pub(crate) const ERROR: Color = hex_to_color(b"#FFB4AB");
    pub(crate) const ON_ERROR: Color = hex_to_color(b"#690005");
    pub(crate) const ERROR_CONTAINER: Color = hex_to_color(b"#93000A");
    pub(crate) const ON_ERROR_CONTAINER: Color = hex_to_color(b"#FFDAD6");
    pub(crate) const BACKGROUND: Color = hex_to_color(b"#15130B");
    pub(crate) const ON_BACKGROUND: Color = hex_to_color(b"#E8E2D4");
    pub(crate) const SURFACE: Color = hex_to_color(b"#15130B");
    pub(crate) const ON_SURFACE: Color = hex_to_color(b"#E8E2D4");
    pub(crate) const SURFACE_VARIANT: Color = hex_to_color(b"#4B4739");
    pub(crate) const ON_SURFACE_VARIANT: Color = hex_to_color(b"#CDC6B4");
    pub(crate) const OUTLINE: Color = hex_to_color(b"#969080");
    pub(crate) const OUTLINE_VARIANT: Color = hex_to_color(b"#4B4739");
    pub(crate) const SHADOW: Color = hex_to_color(b"#000000");
    pub(crate) const SCRIM: Color = hex_to_color(b"#000000");
    pub(crate) const INVERSE_SURFACE: Color = hex_to_color(b"#E8E2D4");
    pub(crate) const INVERSE_ON_SURFACE: Color = hex_to_color(b"#333027");
    pub(crate) const INVERSE_PRIMARY: Color = hex_to_color(b"#6D5E0F");
    pub(crate) const PRIMARY_FIXED: Color = hex_to_color(b"#F8E288");
    pub(crate) const ON_PRIMARY_FIXED: Color = hex_to_color(b"#211B00");
    pub(crate) const PRIMARY_FIXED_DIM: Color = hex_to_color(b"#DAC66F");
    pub(crate) const ON_PRIMARY_FIXED_VARIANT: Color = hex_to_color(b"#534600");
    pub(crate) const SECONDARY_FIXED: Color = hex_to_color(b"#EEE2BC");
    pub(crate) const ON_SECONDARY_FIXED: Color = hex_to_color(b"#201B04");
    pub(crate) const SECONDARY_FIXED_DIM: Color = hex_to_color(b"#D1C6A2");
    pub(crate) const ON_SECONDARY_FIXED_VARIANT: Color = hex_to_color(b"#4D472A");
    pub(crate) const TERTIARY_FIXED: Color = hex_to_color(b"#C4ECCF");
    pub(crate) const ON_TERTIARY_FIXED: Color = hex_to_color(b"#002110");
    pub(crate) const TERTIARY_FIXED_DIM: Color = hex_to_color(b"#A9D0B3");
    pub(crate) const ON_TERTIARY_FIXED_VARIANT: Color = hex_to_color(b"#2B4E38");
    pub(crate) const SURFACE_DIM: Color = hex_to_color(b"#15130B");
    pub(crate) const SURFACE_BRIGHT: Color = hex_to_color(b"#3C3930");
    pub(crate) const SURFACE_CONTAINER_LOWEST: Color = hex_to_color(b"#100E07");
    pub(crate) const SURFACE_CONTAINER_LOW: Color = hex_to_color(b"#1E1C13");
    pub(crate) const SURFACE_CONTAINER: Color = hex_to_color(b"#222017");
    pub(crate) const SURFACE_CONTAINER_HIGH: Color = hex_to_color(b"#2C2A21");
    pub(crate) const SURFACE_CONTAINER_HIGHEST: Color = hex_to_color(b"#37352B");
}

#[allow(dead_code)]
pub(crate) mod shades {
    use crate::consts::colors::hex_to_color;
    use bevy::color::Color;

    pub(crate) const PRIMARY_0: Color = hex_to_color(b"#000000");
    pub(crate) const PRIMARY_5: Color = hex_to_color(b"#151100");
    pub(crate) const PRIMARY_10: Color = hex_to_color(b"#211B00");
    pub(crate) const PRIMARY_15: Color = hex_to_color(b"#2D2500");
    pub(crate) const PRIMARY_20: Color = hex_to_color(b"#393000");
    pub(crate) const PRIMARY_25: Color = hex_to_color(b"#463B00");
    pub(crate) const PRIMARY_30: Color = hex_to_color(b"#534600");
    pub(crate) const PRIMARY_35: Color = hex_to_color(b"#605200");
    pub(crate) const PRIMARY_40: Color = hex_to_color(b"#6E5E00");
    pub(crate) const PRIMARY_50: Color = hex_to_color(b"#8A7600");
    pub(crate) const PRIMARY_60: Color = hex_to_color(b"#A79000");
    pub(crate) const PRIMARY_70: Color = hex_to_color(b"#C4AA1D");
    pub(crate) const PRIMARY_80: Color = hex_to_color(b"#E1C63B");
    pub(crate) const PRIMARY_90: Color = hex_to_color(b"#FFE256");
    pub(crate) const PRIMARY_95: Color = hex_to_color(b"#FFF1BC");
    pub(crate) const PRIMARY_98: Color = hex_to_color(b"#FFF9ED");
    pub(crate) const PRIMARY_99: Color = hex_to_color(b"#FFFBFF");
    pub(crate) const PRIMARY_100: Color = hex_to_color(b"#FFFFFF");
    pub(crate) const SECONDARY_0: Color = hex_to_color(b"#000000");
    pub(crate) const SECONDARY_5: Color = hex_to_color(b"#151100");
    pub(crate) const SECONDARY_10: Color = hex_to_color(b"#211B02");
    pub(crate) const SECONDARY_15: Color = hex_to_color(b"#2C2609");
    pub(crate) const SECONDARY_20: Color = hex_to_color(b"#373013");
    pub(crate) const SECONDARY_25: Color = hex_to_color(b"#423B1D");
    pub(crate) const SECONDARY_30: Color = hex_to_color(b"#4E4727");
    pub(crate) const SECONDARY_35: Color = hex_to_color(b"#5A5232");
    pub(crate) const SECONDARY_40: Color = hex_to_color(b"#665E3D");
    pub(crate) const SECONDARY_50: Color = hex_to_color(b"#807754");
    pub(crate) const SECONDARY_60: Color = hex_to_color(b"#9A916B");
    pub(crate) const SECONDARY_70: Color = hex_to_color(b"#B6AB84");
    pub(crate) const SECONDARY_80: Color = hex_to_color(b"#D2C69E");
    pub(crate) const SECONDARY_90: Color = hex_to_color(b"#EEE2B8");
    pub(crate) const SECONDARY_95: Color = hex_to_color(b"#FDF1C5");
    pub(crate) const SECONDARY_98: Color = hex_to_color(b"#FFF9ED");
    pub(crate) const SECONDARY_99: Color = hex_to_color(b"#FFFBFF");
    pub(crate) const SECONDARY_100: Color = hex_to_color(b"#FFFFFF");
    pub(crate) const TERTIARY_0: Color = hex_to_color(b"#000000");
    pub(crate) const TERTIARY_5: Color = hex_to_color(b"#001508");
    pub(crate) const TERTIARY_10: Color = hex_to_color(b"#002110");
    pub(crate) const TERTIARY_15: Color = hex_to_color(b"#012C18");
    pub(crate) const TERTIARY_20: Color = hex_to_color(b"#0E3822");
    pub(crate) const TERTIARY_25: Color = hex_to_color(b"#1B432C");
    pub(crate) const TERTIARY_30: Color = hex_to_color(b"#274F37");
    pub(crate) const TERTIARY_35: Color = hex_to_color(b"#335B42");
    pub(crate) const TERTIARY_40: Color = hex_to_color(b"#3F674D");
    pub(crate) const TERTIARY_50: Color = hex_to_color(b"#578065");
    pub(crate) const TERTIARY_60: Color = hex_to_color(b"#709A7E");
    pub(crate) const TERTIARY_70: Color = hex_to_color(b"#8AB597");
    pub(crate) const TERTIARY_80: Color = hex_to_color(b"#A5D1B1");
    pub(crate) const TERTIARY_90: Color = hex_to_color(b"#C0EDCD");
    pub(crate) const TERTIARY_95: Color = hex_to_color(b"#CEFCDA");
    pub(crate) const TERTIARY_98: Color = hex_to_color(b"#E9FFEC");
    pub(crate) const TERTIARY_99: Color = hex_to_color(b"#F5FFF4");
    pub(crate) const TERTIARY_100: Color = hex_to_color(b"#FFFFFF");
    pub(crate) const NEUTRAL_0: Color = hex_to_color(b"#000000");
    pub(crate) const NEUTRAL_5: Color = hex_to_color(b"#12110C");
    pub(crate) const NEUTRAL_10: Color = hex_to_color(b"#1D1B16");
    pub(crate) const NEUTRAL_15: Color = hex_to_color(b"#282620");
    pub(crate) const NEUTRAL_20: Color = hex_to_color(b"#32302A");
    pub(crate) const NEUTRAL_25: Color = hex_to_color(b"#3E3B35");
    pub(crate) const NEUTRAL_30: Color = hex_to_color(b"#494740");
    pub(crate) const NEUTRAL_35: Color = hex_to_color(b"#55524B");
    pub(crate) const NEUTRAL_40: Color = hex_to_color(b"#615E57");
    pub(crate) const NEUTRAL_50: Color = hex_to_color(b"#7A776F");
    pub(crate) const NEUTRAL_60: Color = hex_to_color(b"#949088");
    pub(crate) const NEUTRAL_70: Color = hex_to_color(b"#AFABA2");
    pub(crate) const NEUTRAL_80: Color = hex_to_color(b"#CBC6BD");
    pub(crate) const NEUTRAL_90: Color = hex_to_color(b"#E7E2D9");
    pub(crate) const NEUTRAL_95: Color = hex_to_color(b"#F6F0E7");
    pub(crate) const NEUTRAL_98: Color = hex_to_color(b"#FFF9EF");
    pub(crate) const NEUTRAL_99: Color = hex_to_color(b"#FFFBFF");
    pub(crate) const NEUTRAL_100: Color = hex_to_color(b"#FFFFFF");
    pub(crate) const NEUTRAL_VARIANT_0: Color = hex_to_color(b"#000000");
    pub(crate) const NEUTRAL_VARIANT_5: Color = hex_to_color(b"#131107");
    pub(crate) const NEUTRAL_VARIANT_10: Color = hex_to_color(b"#1E1C10");
    pub(crate) const NEUTRAL_VARIANT_15: Color = hex_to_color(b"#29261A");
    pub(crate) const NEUTRAL_VARIANT_20: Color = hex_to_color(b"#343024");
    pub(crate) const NEUTRAL_VARIANT_25: Color = hex_to_color(b"#3F3B2E");
    pub(crate) const NEUTRAL_VARIANT_30: Color = hex_to_color(b"#4B4739");
    pub(crate) const NEUTRAL_VARIANT_35: Color = hex_to_color(b"#565244");
    pub(crate) const NEUTRAL_VARIANT_40: Color = hex_to_color(b"#635E50");
    pub(crate) const NEUTRAL_VARIANT_50: Color = hex_to_color(b"#7C7768");
    pub(crate) const NEUTRAL_VARIANT_60: Color = hex_to_color(b"#969080");
    pub(crate) const NEUTRAL_VARIANT_70: Color = hex_to_color(b"#B1AB9A");
    pub(crate) const NEUTRAL_VARIANT_80: Color = hex_to_color(b"#CDC6B4");
    pub(crate) const NEUTRAL_VARIANT_90: Color = hex_to_color(b"#E9E2D0");
    pub(crate) const NEUTRAL_VARIANT_95: Color = hex_to_color(b"#F8F0DD");
    pub(crate) const NEUTRAL_VARIANT_98: Color = hex_to_color(b"#FFF9ED");
    pub(crate) const NEUTRAL_VARIANT_99: Color = hex_to_color(b"#FFFBFF");
    pub(crate) const NEUTRAL_VARIANT_100: Color = hex_to_color(b"#FFFFFF");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nibble() {
        const HEXDIGITS: [(u8, u8); 22] = [
            (b'0', 0),
            (b'1', 1),
            (b'2', 2),
            (b'3', 3),
            (b'4', 4),
            (b'5', 5),
            (b'6', 6),
            (b'7', 7),
            (b'8', 8),
            (b'9', 9),
            (b'A', 10),
            (b'B', 11),
            (b'C', 12),
            (b'D', 13),
            (b'E', 14),
            (b'F', 15),
            (b'a', 10),
            (b'b', 11),
            (b'c', 12),
            (b'd', 13),
            (b'e', 14),
            (b'f', 15),
        ];

        for (input, expected) in HEXDIGITS {
            assert_eq!(hex_to_nibble(input), expected);
        }
    }

    #[test]
    fn test_channel() {
        const CASES: [(&[u8; 2], u8); 4] = [(b"ff", 255), (b"Ff", 255), (b"bA", 186), (b"a4", 164)];

        for (input, expected) in CASES {
            assert_eq!(hex_to_channel(*input), expected);
        }
    }

    #[test]
    fn test_color() {
        const CASES: [&[u8; 7]; 4] = [b"#2b4e38", b"#100e07", b"#ffb4ab", b"#333027"];

        for case in CASES {
            let u8_channels = [
                hex_to_channel([case[1], case[2]]),
                hex_to_channel([case[3], case[4]]),
                hex_to_channel([case[5], case[6]]),
            ];
            let color = hex_to_color(case).to_srgba();
            assert_eq!(color, Srgba::from_u8_array_no_alpha(u8_channels));
        }
    }
}
