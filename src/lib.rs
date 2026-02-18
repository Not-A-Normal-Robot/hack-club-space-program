pub mod assets;
pub mod builders;
pub mod components;
pub mod consts;
pub mod math;
pub mod plugins;
pub mod resources;
pub mod systems;
pub mod terrain;

/// A wrapper around [`bevy::log::trace!`] that only
/// gets considered when the `trace` feature is enabled.
macro_rules! trace {
    ($( $inner:tt )*) => {
        #[cfg(feature = "trace")]
        ::bevy::log::trace!($( $inner )*);
    };
}

pub(crate) use trace;
