#[macro_export]
macro_rules! fl {
    ($($inner:tt)*) => {
        ::i18n_embed_fl::fl!(*$crate::consts::FLUENT_LANGUAGE_LOADER, $($inner)*)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_fl() {
        let x = ::i18n_embed_fl::fl!(
            crate::consts::FLUENT_LANGUAGE_LOADER,
            "mainMenu__playButton__text"
        );
        let y = fl!("mainMenu__playButton__text");
        assert_eq!(x, y);
    }
}
