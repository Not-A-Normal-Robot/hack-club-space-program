use bevy::{asset::embedded_asset, prelude::*};

pub const LICENSE_DOTO: &str = include_str!("doto/OFL.txt");
pub const LICENSE_WDXL: &str = include_str!("WDXL_Lubrifont_SC/OFL.txt");

macro_rules! define_fonts {
    ($( $name: ident = $rel_path: literal ),* $(,)?) => {
        $(::pastey::paste! {
            pub const [< URI_FONT_ $name >]: &str =
                concat!("embedded://hack_club_space_program/assets/fonts/", $rel_path);
        })*

        pub fn initialize_fonts(app: &mut ::bevy::app::App) {
            app.init_asset::<::bevy::text::Font>();
            $(
                embedded_asset!(app, $rel_path);
            )*
        }
    };
}

define_fonts! {
    DOTO_ROUNDED_BLACK = "doto/Doto_Rounded-Black.ttf",
    DOTO_ROUNDED_BOLD = "doto/Doto_Rounded-Bold.ttf",
    DOTO_BLACK = "doto/Doto-Black.ttf",
    DOTO_BOLD = "doto/Doto-Bold.ttf",
    WDXL_LUBRIFONT_SC = "WDXL_Lubrifont_SC/WDXLLubrifontSC-Regular.ttf",
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{asset::io::embedded::GetAssetServer, text::FontLoader};

    fn min_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app
    }

    #[test]
    fn test_uris() {
        let uris = [
            URI_FONT_DOTO_BLACK,
            URI_FONT_DOTO_BOLD,
            URI_FONT_DOTO_ROUNDED_BLACK,
            URI_FONT_DOTO_ROUNDED_BOLD,
            URI_FONT_WDXL_LUBRIFONT_SC,
        ];
        let mut app = min_app();
        app.get_asset_server().register_loader(FontLoader);

        initialize_fonts(&mut app);

        let server = app.get_asset_server();

        let invalid_handle = server.load::<Font>("embedded://erm/this/doesnt/exist/haha.otf");
        let handles = uris.map(|uri| server.load::<Font>(uri));

        for _ in 0..8192 {
            app.update();
        }

        let server = app.get_asset_server();

        let state = server.load_state(invalid_handle.id().untyped());
        assert!(state.is_failed(), "Expected {state:?} to be failed");

        for handle in handles {
            let state = server.load_state(handle.id().untyped());
            assert!(state.is_loaded(), "Expected {state:?} to be loaded");
        }
    }
}
