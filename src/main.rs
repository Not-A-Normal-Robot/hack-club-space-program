use bevy::{pbr::wireframe::WireframeConfig, prelude::*};
use hack_club_space_program::plugins::setup::GameSetupPlugin;

fn enable_backtrace() {
    const BACKTRACE_KEY: &str = "RUST_BACKTRACE";
    unsafe {
        if std::env::var(BACKTRACE_KEY).is_err() {
            std::env::set_var(BACKTRACE_KEY, "1");
        }
    }
}

fn main() {
    enable_backtrace();

    #[cfg(target_family = "wasm")]
    hack_club_space_program::web::init_panic_handler();

    App::new()
        .add_plugins(GameSetupPlugin)
        .insert_resource(WireframeConfig {
            global: true,
            default_color: Color::Srgba(Srgba::RED),
        })
        .run();
}
