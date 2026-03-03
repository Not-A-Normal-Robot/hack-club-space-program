macro_rules! define_icons {
    ($( $name: ident = $rel_path: literal ),* $(,)?) => {
        $(::pastey::paste! {
            #[allow(dead_code)]
            pub(crate) const [< URI_ICON_ $name >]: &str =
                concat!("embedded://hack_club_space_program/assets/icons/", $rel_path);
        })*

        pub(super) fn initialize_icons(app: &mut ::bevy::app::App) {
            app.add_plugins(::bevy_vello::VelloPlugin::default());
        }
    };
}

define_icons! {
    PROGRADE = "prograde.svg",
}
