#[macro_export]
macro_rules! fl {
    ($($inner:tt)*) => {
        ::i18n_embed_fl::fl!(*$crate::consts::FLUENT_LANGUAGE_LOADER, $($inner)*)
    };
}

/// Assign to a variable if they're not equal.
///
/// Wrapper for:
/// ```
/// # let mut lhs = 1;
/// # let rhs = 2;
/// if lhs != rhs {
///     lhs = rhs
/// }
/// ```
///
/// This is useful for things that track mutable assignments
/// but don't check equality.
#[macro_export]
macro_rules! checked_assign {
    ($lhs:expr, $rhs:expr) => {
        if $lhs != $rhs {
            $lhs = $rhs;
        }
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

    #[test]
    fn test_checked_assign() {
        let mut lhs = 1;
        let rhs = 2;

        checked_assign!(lhs, rhs);
        assert_eq!(lhs, rhs);
    }
}
