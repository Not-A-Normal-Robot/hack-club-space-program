pub(crate) mod assets;
pub mod builders;
pub mod components;
pub mod consts;
pub(crate) mod macros;
pub(crate) mod math;
pub mod plugins;
pub mod resources;
pub(crate) mod systems;
pub(crate) mod terrain;
#[cfg(target_family = "wasm")]
pub mod web;

/// A wrapper around [`bevy::log::trace!`] that only
/// gets considered when the `trace` feature is enabled.
macro_rules! trace {
    ($( $inner:tt )*) => {
        #[cfg(feature = "trace")]
        ::bevy::log::trace!($( $inner )*);
    };
}

pub(crate) use trace;
