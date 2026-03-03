macro_rules! define_icons {
    ($( $name: ident = $rel_path: literal ),* $(,)?) => {
        $(::pastey::paste! {
            #[allow(dead_code)]
            pub(crate) const [< URI_ICON_ $name >]: &str =
                concat!("embedded://hack_club_space_program/assets/icons/", $rel_path);
        })*

        pub(super) fn initialize_icons(app: &mut ::bevy::app::App) {
            // TODO: Initialize Vello
            // <::bevy::app::App as ::bevy::asset::AssetApp>
            //     ::init_asset::<::bevy_resvg::raster::asset::SvgFile>(app);
            // $(
            //     ::bevy::asset::embedded_asset!(app, $rel_path);
            // )*
        }
    };
}

define_icons! {
    PROGRADE = "prograde.svg",
}
