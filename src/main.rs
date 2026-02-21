use bevy::prelude::*;
use hack_club_space_program::plugins::setup::GameSetupPlugin;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(not(target_family = "wasm"))]
fn enable_backtrace() {
    const BACKTRACE_KEY: &str = "RUST_BACKTRACE";
    unsafe {
        if std::env::var(BACKTRACE_KEY).is_err() {
            std::env::set_var(BACKTRACE_KEY, "1");
        }
    }
}

fn main() {
    #[cfg(not(target_family = "wasm"))]
    enable_backtrace();

    App::new().add_plugins(GameSetupPlugin).run();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen(start)]
fn web_start() {
    hack_club_space_program::web::panic_handler::init_panic_handler();
    main();
}
