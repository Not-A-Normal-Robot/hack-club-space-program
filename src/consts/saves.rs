//! Save data structures and the like.
//!
//! # BEWARE OF THE PIPELINE
//! `TOML` --serde-> `UnvalidatedSaveData` --validation-> `ValidatedSaveData`

use bevy::{platform::collections::HashMap, prelude::*};
use core::fmt::Display;
use derive_more::{Deref, DerefMut, Error};
use keplerian_sim::{CompactOrbit2D, Orbit2D};
use serde::{Deserialize, Serialize, de::Visitor};
use std::{borrow::Cow, fmt::Write};

use crate::{
    components::main_game::{
        celestial::Terrain,
        relations::{RailMode, SurfaceAttachment},
    },
    consts::GRAVITATIONAL_CONSTANT,
    fl,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, DerefMut)]
#[repr(transparent)]
pub struct SavedId(#[deref] pub u128);

impl SavedId {
    /// The amount of characters the saved ID takes to represent itself
    /// as a hexadecimal-encoded string.
    pub const CHARS: u32 = u128::BITS / 4;
}

impl From<u128> for SavedId {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<SavedId> for u128 {
    fn from(value: SavedId) -> Self {
        value.0
    }
}

impl Display for SavedId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl Serialize for SavedId {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:032x}", self.0))
    }
}

impl<'de> Deserialize<'de> for SavedId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IdVisitor;

        impl Visitor<'_> for IdVisitor {
            type Value = SavedId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "a hexadecimal u128 value with {} bytes/ASCII bytes",
                    Self::Value::CHARS
                )
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.len() != Self::Value::CHARS as usize {
                    return Err(serde::de::Error::invalid_length(v.len(), &self));
                }

                u128::from_str_radix(v, 16)
                    .map(SavedId)
                    .map_err(|e| serde::de::Error::custom(e))
            }
        }

        deserializer.deserialize_str(IdVisitor)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct RawSaveData {
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // ================== MEMO ==================
    //  DO NOT FORGET TO UPDATE THE SCHEMA FILE
    //        AT `./save_data.schema.json`
    //         IF YOU ARE EDITING THIS!
    // ==========================================
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    /// The celestial body identifier to use as the root node.
    pub(crate) root_node: SavedId,
    /// The vessel identifier to use as the active vessel.
    pub(crate) active_vessel: SavedId,
    /// A map of celestial bodies.
    ///
    /// If this body can get its orbital data from the NASA Horizons API,
    /// use a hex representation of its Horizons ID.
    ///
    /// If it shouldn't, use an ID of greater than `0xFFFF_FFFF` to skip it.
    pub(crate) celestials: HashMap<SavedId, CelestialData>,
    /// A map of vessels.
    pub(crate) vessels: HashMap<SavedId, VesselData>,
}

impl RawSaveData {
    fn validate(&self) -> Result<(), SaveDataError> {
        if !self.celestials.contains_key(&self.root_node) {
            return Err(SaveDataError::RootCelestialNotFound);
        }

        if !self.vessels.contains_key(&self.active_vessel) {
            return Err(SaveDataError::ActiveVesselNotFound);
        }

        let mut to_traverse = vec![self.root_node];

        // Maps between celestial objects and its parent
        let mut celestial_map: HashMap<SavedId, Option<SavedId>> =
            HashMap::with_capacity(self.celestials.len());
        let mut vessel_map: HashMap<SavedId, SavedId> = HashMap::with_capacity(self.vessels.len());
        celestial_map.insert(self.root_node, None);

        while let Some(cel_id) = to_traverse.pop() {
            let celestial = self.celestials.get(&cel_id).ok_or_else(|| {
                celestial_map.get(&cel_id).copied().flatten().map_or(
                    SaveDataError::RootCelestialNotFound,
                    |referrer| SaveDataError::CelestialNotFound {
                        referrer,
                        not_found: cel_id,
                    },
                )
            })?;

            for child in &celestial.celestial_children {
                if let Some(first_referrer) = celestial_map.get(child).copied() {
                    return Err(SaveDataError::DuplicateCelestial {
                        duplicated: *child,
                        first_referrer,
                        second_referrer: cel_id,
                    });
                }
                celestial_map.insert(*child, Some(cel_id));
            }

            to_traverse.extend_from_slice(&celestial.celestial_children);

            for vessel in &celestial.vessel_children {
                if !self.vessels.contains_key(vessel) {
                    return Err(SaveDataError::VesselNotFound {
                        not_found: *vessel,
                        referrer: cel_id,
                    });
                }
                if let Some(parent) = vessel_map.get(vessel).copied() {
                    return Err(SaveDataError::DuplicateVessel {
                        duplicated: *vessel,
                        first_referrer: parent,
                        second_referrer: cel_id,
                    });
                }
                vessel_map.insert(*vessel, cel_id);
            }
        }

        if celestial_map.len() != self.celestials.len() {
            let orphans: Box<[SavedId]> = self
                .celestials
                .keys()
                .copied()
                .filter(|id| !celestial_map.contains_key(id))
                .collect();
            return Err(SaveDataError::OrphanedCelestials(orphans));
        }

        if vessel_map.len() != self.vessels.len() {
            let orphans: Box<[SavedId]> = self
                .vessels
                .keys()
                .copied()
                .filter(|id| !vessel_map.contains_key(id))
                .collect();
            return Err(SaveDataError::OrphanedVessels(orphans));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct UnvalidatedSaveData(RawSaveData);

impl UnvalidatedSaveData {
    pub fn validate(self) -> Result<ValidatedSaveData, SaveDataError> {
        self.0.validate().map(|()| ValidatedSaveData(self.0))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedSaveData(RawSaveData);

/// An error indicating an invalid save data.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum SaveDataError {
    /// The `root_node` didn't resolve to any celestial.
    RootCelestialNotFound,
    /// The `active_vessel` didn't resolve to any vessel.
    ActiveVesselNotFound,
    /// There was a reference to a nonexistent celestial.
    CelestialNotFound {
        /// The parent celestial body with the invalid reference.
        referrer: SavedId,
        /// The celestial body ID which wasn't found.
        not_found: SavedId,
    },
    /// There was a reference to a nonexistent vessel.
    VesselNotFound {
        /// The parent celestial body with the invalid reference.
        referrer: SavedId,
        /// The vessel ID which wasn't found.
        not_found: SavedId,
    },
    /// There was a duplicate celestial body in the universe tree.
    DuplicateCelestial {
        /// The celestial body ID which appeared more than once
        /// in the universe tree.
        duplicated: SavedId,
        /// The celestial body ID which referred to this one
        /// for the first time, or [`None`] if this is the
        /// root node.
        first_referrer: Option<SavedId>,
        /// The celestial body ID which referred to this one
        /// for the second time.
        second_referrer: SavedId,
    },
    /// There was a duplicate vessel in the universe tree.
    DuplicateVessel {
        /// The vessel ID which appeared more than once in the
        /// universe tree.
        duplicated: SavedId,
        /// The celestial body ID which referred to this
        /// vessel for the first time.
        first_referrer: SavedId,
        /// The celestial body ID which referred to this
        /// vessel for the second time.
        second_referrer: SavedId,
    },
    /// There were some orphaned celestials.
    OrphanedCelestials(#[error(ignore)] Box<[SavedId]>),
    /// There were some orphaned vessels.
    OrphanedVessels(#[error(ignore)] Box<[SavedId]>),
}

impl Display for SaveDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RootCelestialNotFound => {
                write!(f, "{}", fl!("error__saveData__rootCelestialNotFound"))
            }
            Self::ActiveVesselNotFound => {
                write!(f, "{}", fl!("error__saveData__activeVesselNotFound"))
            }
            Self::CelestialNotFound {
                referrer,
                not_found,
            } => write!(
                f,
                "{}",
                fl!(
                    "error__saveData__celestialNotFound",
                    referrer = referrer.to_string(),
                    not_found = not_found.to_string()
                )
            ),
            Self::VesselNotFound {
                referrer,
                not_found,
            } => write!(
                f,
                "{}",
                fl!(
                    "error__saveData__vesselNotFound",
                    referrer = referrer.to_string(),
                    not_found = not_found.to_string()
                )
            ),
            Self::DuplicateCelestial {
                duplicated,
                first_referrer,
                second_referrer,
            } => write!(
                f,
                "{}",
                fl!(
                    "error__saveData__duplicateCelestial",
                    duplicated = duplicated.to_string(),
                    first_referrer = first_referrer
                        .map_or(Cow::Borrowed("none"), |id| Cow::Owned(id.to_string())),
                    second_referrer = second_referrer.to_string(),
                )
            ),
            Self::DuplicateVessel {
                duplicated,
                first_referrer,
                second_referrer,
            } => write!(
                f,
                "{}",
                fl!(
                    "error__saveData__duplicateVessel",
                    duplicated = duplicated.to_string(),
                    first_referrer = first_referrer.to_string(),
                    second_referrer = second_referrer.to_string(),
                )
            ),
            Self::OrphanedCelestials(saved_ids) => {
                let mut list = String::with_capacity((34 * saved_ids.len()).min(2));
                list.push('[');

                for id in saved_ids.iter().take(saved_ids.len() - 1) {
                    write!(&mut list, "{id}, ").expect("formatting should work");
                }
                if let Some(id) = saved_ids.last() {
                    write!(&mut list, "{id}]").expect("formatting should work");
                }

                write!(
                    f,
                    "{}",
                    fl!("error__saveData__orphanedCelestials", list = list)
                )
            }
            Self::OrphanedVessels(saved_ids) => {
                let mut list = String::with_capacity((34 * saved_ids.len()).min(2));
                list.push('[');

                for id in saved_ids.iter().take(saved_ids.len() - 1) {
                    write!(&mut list, "{id}, ").expect("formatting should work");
                }
                if let Some(id) = saved_ids.last() {
                    write!(&mut list, "{id}]").expect("formatting should work");
                }

                write!(
                    f,
                    "{}",
                    fl!("error__saveData__orphanedVessels", list = list)
                )
            }
        }
    }
}

/// Holds static information about celestial bodies.
///
/// Should be in sync with `./save_data.schema.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CelestialData {
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // ================== MEMO ==================
    //  DO NOT FORGET TO UPDATE THE SCHEMA FILE
    //        AT `./save_data.schema.json`
    //         IF YOU ARE EDITING THIS!
    // ==========================================
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    /// The name of this celestial body.
    name: String,
    /// The mass of this celestial body, in kilograms.
    mass: f64,
    /// The radius of this celestial body, in metres.
    radius: f64,
    /// The color of this celestial body.
    color: Color,
    /// Information about this celestial body's orbital
    /// parameters, if any.
    orbit: Option<OrbitalData>,
    /// A list of this celestial body's celestial children's IDs.
    celestial_children: Box<[SavedId]>,
    /// A list of this celestial body's vessel children's IDs.
    vessel_children: Box<[SavedId]>,
    /// This celestial body's terrain parameters.
    terrain: Option<Terrain>,
}

/// Holds static information about celestial bodies' orbits.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrbitalData {
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // ================== MEMO ==================
    //  DO NOT FORGET TO UPDATE THE SCHEMA FILE
    //        AT `./save_data.schema.json`
    //         IF YOU ARE EDITING THIS!
    // ==========================================
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    /// The ecccentricity of this celestial body's
    /// orbit.
    pub eccentricity: f64,
    /// The periapsis of this celestial body's orbit,
    /// in metres.
    pub periapsis: f64,
    /// The argument of periapsis of this celestial
    /// body's orbit, in radians.
    pub arg_pe: f64,
    /// The mean anomaly at epoch of this celestial
    /// body's orbit, in radians.
    pub mean_anomaly: f64,
}

impl OrbitalData {
    #[inline]
    #[must_use]
    pub const fn to_compact_orbit(self, parent_mass: f64) -> CompactOrbit2D {
        CompactOrbit2D {
            eccentricity: self.eccentricity,
            periapsis: self.eccentricity,
            arg_pe: self.arg_pe,
            mean_anomaly: self.mean_anomaly,
            mu: parent_mass * GRAVITATIONAL_CONSTANT,
        }
    }

    #[inline]
    #[must_use]
    pub fn to_cached_orbit(self, parent_mass: f64) -> Orbit2D {
        self.to_compact_orbit(parent_mass).into()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VesselData {
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // ================== MEMO ==================
    //  DO NOT FORGET TO UPDATE THE SCHEMA FILE
    //        AT `./save_data.schema.json`
    //         IF YOU ARE EDITING THIS!
    // ==========================================
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    /// The (non-unique) name that this vessel is assigned.
    name: String,
    /// The rail that this vessel is on.
    rail: RailData,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "rail_mode", content = "inner")]
#[serde(rename_all = "snake_case")]
pub enum RailData {
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // ================== MEMO ==================
    //  DO NOT FORGET TO UPDATE THE SCHEMA FILE
    //        AT `./save_data.schema.json`
    //         IF YOU ARE EDITING THIS!
    // ==========================================
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    Orbit(OrbitalData),
    Surface(SurfaceAttachment),
}

impl RailData {
    pub fn instantiate(&self, parent_mass: f64) -> RailMode {
        match self {
            Self::Orbit(o) => RailMode::Orbit(o.to_cached_orbit(parent_mass)),
            Self::Surface(s) => RailMode::Surface(*s),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::plugins::i18n::load_localizations;

    use super::*;
    use bevy::platform::collections::HashSet;
    use rand::{RngExt, SeedableRng};
    use serde_test::{Token, assert_de_tokens, assert_de_tokens_error};

    // Tests for the custom SavedId struct's ser/de

    #[test]
    fn test_length_fail() {
        assert_de_tokens_error::<SavedId>(
            &[Token::Str("0000000000000000000000000000000")],
            "invalid length 31, expected a hexadecimal u128 value with 32 bytes/ASCII bytes",
        );
        assert_de_tokens(
            &SavedId(0),
            &[Token::Str("00000000000000000000000000000000")],
        );
        assert_de_tokens_error::<SavedId>(
            &[Token::Str("000000000000000000000000000000000")],
            "invalid length 33, expected a hexadecimal u128 value with 32 bytes/ASCII bytes",
        );
    }

    #[test]
    fn test_bounds() {
        assert_de_tokens(
            &SavedId(u128::MAX),
            &[Token::Str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")],
        );
        assert_de_tokens(
            &SavedId(u128::MAX),
            &[Token::Str("ffffffffffffffffffffffffffffffff")],
        );
    }

    #[test]
    fn test_roundtrip() {
        const ITERS: usize = 100_000;

        let mut rng = rand::rngs::Xoshiro256PlusPlus::seed_from_u64(12_345_678_901_234_567_890);

        for _ in 0..ITERS {
            let id: u128 = rng.random();
            let id = SavedId(id);

            let string = serde_json::to_string(&id).expect("serialization should work");

            assert_eq!(&string, &format!("\"{:032x}\"", id.0));

            let parsed: SavedId =
                serde_json::from_str(&string).expect("deserialization should work");

            assert_eq!(id, parsed);
        }
    }

    fn assert_eq_l10n<S: AsRef<str>>(lhs: S, rhs: S) {
        let [lhs, rhs] = [lhs, rhs].map(|s| {
            s.as_ref()
                .chars()
                .filter(|c| !['\u{2068}', '\u{2069}'].contains(c))
                .collect::<String>()
        });
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn save_data_error_l10n() {
        let [referrer, referrer_2, subject] = [
            0x4bfc_85f1_3889_e1a3_d876_e402_f0c5_6970,
            0x99d7_462a_2c4d_120e_dbcd_38c3_f0a5_890b,
            0x3cc2_a87c_5557_28bb_001c_ae59_8635_4a2c,
        ]
        .map(SavedId);
        let [str_referrer, str_referrer_2, str_subject] =
            [referrer, referrer_2, subject].map(|id| id.to_string());

        load_localizations();

        assert_eq_l10n(
            SaveDataError::CelestialNotFound {
                referrer,
                not_found: subject,
            }
            .to_string(),
            format!(
                "Celestial body with ID {str_referrer} had a reference to a nonexistent celestial body with ID {str_subject}."
            ),
        );

        assert_eq_l10n(
            SaveDataError::VesselNotFound {
                referrer,
                not_found: subject,
            }
            .to_string(),
            format!(
                "Celestial body with ID {str_referrer} had a reference to a nonexistent vessel with ID {str_subject}."
            ),
        );

        assert_eq_l10n(
            SaveDataError::DuplicateCelestial {
                duplicated: subject,
                first_referrer: Some(referrer),
                second_referrer: referrer_2,
            }
            .to_string(),
            format!(
                "Celestial body with ID {str_subject} appears more than once in references: \
                it's referenced as a child of celestial bodies with IDs {str_referrer} and {str_referrer_2}."
            ),
        );

        assert_eq_l10n(
            SaveDataError::DuplicateCelestial {
                duplicated: subject,
                first_referrer: None,
                second_referrer: referrer_2,
            }
            .to_string(),
            format!(
                "Celestial body with ID {str_subject} appears more than once in references: \
                it's referenced as the root element and the child of the celestial body with ID {str_referrer_2}."
            ),
        );

        assert_eq_l10n(
            SaveDataError::DuplicateVessel {
                duplicated: subject,
                first_referrer: referrer,
                second_referrer: referrer_2,
            }
            .to_string(),
            format!(
                "Vessel with ID {str_subject} appears more than once in references: \
                it's referenced as a child of celestial bodies with IDs {str_referrer} and {str_referrer_2}."
            ),
        );

        assert_eq_l10n(
            SaveDataError::OrphanedCelestials(Box::new([referrer, referrer_2])).to_string(),
            format!(
                "The save file contains celestial bodies without a parent: [{str_referrer}, {str_referrer_2}]"
            ),
        );

        assert_eq_l10n(
            SaveDataError::OrphanedVessels(Box::new([referrer, referrer_2])).to_string(),
            format!(
                "The save file contains vessels without a parent: [{str_referrer}, {str_referrer_2}]"
            ),
        );
    }

    fn gen_celestial_data() -> CelestialData {
        CelestialData {
            name: String::new(),
            mass: 9.0,
            radius: 10.0,
            color: Color::WHITE,
            orbit: None,
            celestial_children: Box::new([]),
            vessel_children: Box::new([]),
            terrain: None,
        }
    }

    fn gen_vessel_data() -> VesselData {
        VesselData {
            name: String::new(),
            rail: gen_rail_data(),
        }
    }

    fn gen_rail_data() -> RailData {
        RailData::Orbit(gen_orbital_data())
    }

    fn gen_orbital_data() -> OrbitalData {
        OrbitalData {
            eccentricity: 0.0,
            periapsis: 100.0,
            arg_pe: 1.0,
            mean_anomaly: 2.0,
        }
    }

    #[test]
    #[expect(clippy::too_many_lines)]
    fn save_data_validation() {
        let root_not_found = UnvalidatedSaveData(RawSaveData {
            root_node: SavedId(0),
            active_vessel: SavedId(0),
            celestials: [(
                SavedId(1),
                CelestialData {
                    vessel_children: Box::from([SavedId(0)]),
                    ..gen_celestial_data()
                },
            )]
            .into(),
            vessels: [(SavedId(0), gen_vessel_data())].into(),
        })
        .validate();
        assert_eq!(root_not_found, Err(SaveDataError::RootCelestialNotFound));

        let active_vessel_not_found = UnvalidatedSaveData(RawSaveData {
            root_node: SavedId(0),
            active_vessel: SavedId(0),
            celestials: [(
                SavedId(0),
                CelestialData {
                    vessel_children: Box::from([SavedId(1)]),
                    ..gen_celestial_data()
                },
            )]
            .into(),
            vessels: [(SavedId(1), gen_vessel_data())].into(),
        })
        .validate();
        assert_eq!(
            active_vessel_not_found,
            Err(SaveDataError::ActiveVesselNotFound)
        );

        let celestial_not_found = UnvalidatedSaveData(RawSaveData {
            root_node: SavedId(200),
            active_vessel: SavedId(0),
            celestials: [(
                SavedId(200),
                CelestialData {
                    celestial_children: Box::from([SavedId(404)]),
                    vessel_children: Box::from([SavedId(0)]),
                    ..gen_celestial_data()
                },
            )]
            .into(),
            vessels: [(SavedId(0), gen_vessel_data())].into(),
        })
        .validate();
        assert_eq!(
            celestial_not_found,
            Err(SaveDataError::CelestialNotFound {
                referrer: SavedId(200),
                not_found: SavedId(404)
            })
        );

        let vessel_not_found = UnvalidatedSaveData(RawSaveData {
            root_node: SavedId(200),
            active_vessel: SavedId(0),
            celestials: [(
                SavedId(200),
                CelestialData {
                    vessel_children: Box::from([SavedId(0), SavedId(404)]),
                    ..gen_celestial_data()
                },
            )]
            .into(),
            vessels: [(SavedId(0), gen_vessel_data())].into(),
        })
        .validate();
        assert_eq!(
            vessel_not_found,
            Err(SaveDataError::VesselNotFound {
                referrer: SavedId(200),
                not_found: SavedId(404)
            }),
        );

        let duplicate_root_celestial = UnvalidatedSaveData(RawSaveData {
            root_node: SavedId(0),
            active_vessel: SavedId(0),
            celestials: [(
                SavedId(0),
                CelestialData {
                    vessel_children: Box::from([SavedId(0)]),
                    celestial_children: Box::from([SavedId(0)]),
                    ..gen_celestial_data()
                },
            )]
            .into(),
            vessels: [(SavedId(0), gen_vessel_data())].into(),
        })
        .validate();
        assert_eq!(
            duplicate_root_celestial,
            Err(SaveDataError::DuplicateCelestial {
                duplicated: SavedId(0),
                first_referrer: None,
                second_referrer: SavedId(0)
            }),
        );

        let duplicate_celestial = UnvalidatedSaveData(RawSaveData {
            root_node: SavedId(0),
            active_vessel: SavedId(0),
            celestials: [
                (
                    SavedId(0),
                    CelestialData {
                        vessel_children: Box::from([SavedId(0)]),
                        celestial_children: Box::from([SavedId(1), SavedId(2)]),
                        ..gen_celestial_data()
                    },
                ),
                (SavedId(1), gen_celestial_data()),
                (
                    SavedId(2),
                    CelestialData {
                        celestial_children: Box::from([SavedId(1)]),
                        ..gen_celestial_data()
                    },
                ),
            ]
            .into(),
            vessels: [(SavedId(0), gen_vessel_data())].into(),
        })
        .validate()
        .unwrap_err();
        match duplicate_celestial {
            SaveDataError::DuplicateCelestial {
                duplicated,
                first_referrer,
                second_referrer,
            } => {
                assert_eq!(duplicated, SavedId(1));
                let first_referrer = first_referrer.unwrap();
                let error_referrers = HashSet::from([first_referrer, second_referrer]);
                let expected_referrers = HashSet::from([SavedId(0), SavedId(2)]);
                assert_eq!(error_referrers, expected_referrers);
            }
            _ => panic!(
                "duplicate_celestial ({duplicate_celestial:?}) is not of variant DuplicateCelestial"
            ),
        }

        let duplicate_vessel = UnvalidatedSaveData(RawSaveData {
            root_node: SavedId(0),
            active_vessel: SavedId(0),
            celestials: [
                (
                    SavedId(0),
                    CelestialData {
                        celestial_children: Box::from([SavedId(1)]),
                        vessel_children: Box::from([SavedId(0)]),
                        ..gen_celestial_data()
                    },
                ),
                (
                    SavedId(1),
                    CelestialData {
                        vessel_children: Box::from([SavedId(0)]),
                        ..gen_celestial_data()
                    },
                ),
            ]
            .into(),
            vessels: [(SavedId(0), gen_vessel_data())].into(),
        })
        .validate()
        .unwrap_err();
        match duplicate_vessel {
            SaveDataError::DuplicateVessel {
                duplicated,
                first_referrer,
                second_referrer,
            } => {
                assert_eq!(duplicated, SavedId(0));
                let error_referrers = HashSet::from([first_referrer, second_referrer]);
                let expected_referrers = HashSet::from([SavedId(0), SavedId(1)]);
                assert_eq!(error_referrers, expected_referrers);
            }
            _ => {
                panic!("duplicate_vessel ({duplicate_vessel:?}) is not of variant DuplicateVessel")
            }
        }

        let orphaned_celestials = UnvalidatedSaveData(RawSaveData {
            root_node: SavedId(0),
            active_vessel: SavedId(0),
            celestials: [
                (
                    SavedId(0),
                    CelestialData {
                        vessel_children: Box::from([SavedId(0)]),
                        ..gen_celestial_data()
                    },
                ),
                (SavedId(1), gen_celestial_data()),
                (SavedId(2), gen_celestial_data()),
            ]
            .into(),
            vessels: [(SavedId(0), gen_vessel_data())].into(),
        })
        .validate();

        let Err(SaveDataError::OrphanedCelestials(orphans)) = orphaned_celestials else {
            panic!(
                "Expected {orphaned_celestials:?} to be of variant Err(SaveDataError::OrphanedCelestials)"
            );
        };

        assert_eq!(
            orphans.into_iter().collect::<HashSet<_>>(),
            HashSet::from([SavedId(1), SavedId(2)])
        );

        let orphaned_vessels = UnvalidatedSaveData(RawSaveData {
            root_node: SavedId(0),
            active_vessel: SavedId(0),
            celestials: [(
                SavedId(0),
                CelestialData {
                    vessel_children: Box::from([SavedId(0)]),
                    ..gen_celestial_data()
                },
            )]
            .into(),
            vessels: [
                (SavedId(0), gen_vessel_data()),
                (SavedId(1), gen_vessel_data()),
                (SavedId(2), gen_vessel_data()),
            ]
            .into(),
        })
        .validate();

        let Err(SaveDataError::OrphanedVessels(orphans)) = orphaned_vessels else {
            panic!(
                "Expected {orphaned_vessels:?} to be of variant Err(SaveDataError::OrphanedVessels)"
            );
        };

        assert_eq!(
            orphans.into_iter().collect::<HashSet<_>>(),
            HashSet::from([SavedId(1), SavedId(2)])
        );

        UnvalidatedSaveData(RawSaveData {
            root_node: SavedId(0),
            active_vessel: SavedId(0),
            celestials: [(
                SavedId(0),
                CelestialData {
                    vessel_children: Box::from([SavedId(0)]),
                    ..gen_celestial_data()
                },
            )]
            .into(),
            vessels: [(SavedId(0), gen_vessel_data())].into(),
        })
        .validate()
        .unwrap();
    }
}
