use bevy::prelude::*;
use strum::EnumCount;

use crate::resources::scene::GameScene;

/// The reference frame that this altimeter is using for its measurements.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumCount, Hash, SubStates)]
#[source(GameScene = GameScene::InGame)]
pub(crate) enum AltimeterMode {
    #[default]
    AboveSeaLevel,
    AboveGroundLevel,
    FromCentre,
}

impl AltimeterMode {
    /// Get the next mode to be used when the user
    /// requests it.
    #[must_use]
    pub(crate) fn next(self) -> Self {
        match self {
            Self::AboveSeaLevel => Self::AboveGroundLevel,
            Self::AboveGroundLevel => Self::FromCentre,
            Self::FromCentre => Self::AboveSeaLevel,
        }
    }
}

#[cfg(test)]
mod test {
    use bevy::platform::collections::HashSet;
    use strum::EnumCount;

    use crate::resources::ui::AltimeterMode;

    #[test]
    fn altimeter_mode_traverse() {
        let count = AltimeterMode::COUNT;

        let mut set = HashSet::with_capacity(count);
        let mut mode = AltimeterMode::default();

        for _ in 0..count {
            set.insert(mode);
            mode = mode.next();
        }

        assert_eq!(set.len(), count);
    }
}
