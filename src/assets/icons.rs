macro_rules! define_icons {
    ($( $name: ident = $rel_path: literal ),* $(,)?) => {
        $(::pastey::paste! {
            #[allow(dead_code)]
            pub(crate) const [< URI_ICON_ $name >]: &str =
                concat!("embedded://hack_club_space_program/assets/../../assets/_processed/icons/", $rel_path);
        })*

        pub(super) fn initialize_icons(app: &mut ::bevy::app::App) {
            $(
                ::bevy::asset::embedded_asset!(app, concat!("../../assets/_processed/icons/", $rel_path));
            )*
        }
    };
}

define_icons! {
    PROGRADE = "prograde.png",
    RETROGRADE = "retrograde.png",
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        asset::io::embedded::GetAssetServer,
        image::{CompressedImageFormats, ImageLoader},
        prelude::*,
    };

    fn min_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app
    }

    #[test]
    fn test_uris() {
        let uris = [URI_ICON_PROGRADE];
        let mut app = min_app();
        app.init_asset::<Image>();
        app.get_asset_server()
            .register_loader(ImageLoader::new(CompressedImageFormats::all()));

        initialize_icons(&mut app);

        let server = app.get_asset_server();

        let invalid_handle = server.load::<Image>("embedded://erm/this/doesnt/exist/haha.otf");
        let handles = uris.map(|uri| server.load::<Image>(uri));

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
