use crate::fl;
use core::fmt::Display;
use std::{io, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum RiskyPathReason {
    /// We failed to canonicalize the path, so
    /// we don't know if it's dangerous
    CanonicalizeFailure(#[from] io::Error),
    /// We don't recognize this platform, so
    /// we don't know if it's dangerous
    #[allow(dead_code)]
    UnknownPlatform,
    /// This path leads to a root directory
    RootDirectory,
    /// This path leads to a critical non-root directory
    CriticalDirectory,
    /// We don't know this path structure, so
    /// we don't know if it's dangerous
    UnknownPath,
}

impl Display for RiskyPathReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CanonicalizeFailure(inner) => f.write_str(&fl!(
                "error__riskyPath__canonFailure",
                inner = inner.to_string()
            )),
            Self::UnknownPlatform => f.write_str(&fl!("error__riskyPath__unknownPlatform")),
            Self::RootDirectory => f.write_str(&fl!("error__riskyPath__rootDir")),
            Self::CriticalDirectory => f.write_str(&fl!("error__riskyPath__critDir")),
            Self::UnknownPath => f.write_str(&fl!("error__riskyPath__unknownPath")),
        }
    }
}

/// Returns `Err(RiskyPathReason)` if the path was deemed risky,
/// else returns `None`.
pub(super) fn check_path_risk(path: &Path) -> Result<(), RiskyPathReason> {
    let path = path.canonicalize()?;
    if cfg!(unix) {
        return check_unix_path_risk(&path);
    }
    if cfg!(windows) {
        return check_windows_path_risk(&path);
    }

    Err(RiskyPathReason::UnknownPlatform)
}

/// # Unchecked Operation
/// This function does not check whether or not the given path is already
/// canonicalized. Please make sure to canonicalize the path beforehand.
fn check_unix_path_risk(canon_path: &Path) -> Result<(), RiskyPathReason> {
    if canon_path == "/" {
        return Err(RiskyPathReason::RootDirectory);
    }

    // These dirs are critical, but not their contents
    let critical_exact_dirs = [
        "/home", "/mnt",     // FDS
        "/Users",   // Mac
        "/storage", // Android
    ];

    // These dirs are critical along with their contents
    let critical_prefix_dirs = [
        // FDS
        "/usr",
        "/etc",
        "/boot",
        "/bin",
        "/dev",
        "/lib",
        "/lib64",
        "/lost+found",
        "/media",
        "/opt",
        "/root",
        "/sbin",
        "/srv",
        "/sys",
        "/var",
        // Mac: none currently
        // Android
        "/system",
    ];

    for crit in critical_exact_dirs {
        if canon_path == crit {
            return Err(RiskyPathReason::CriticalDirectory);
        }
    }

    for crit in critical_prefix_dirs {
        if canon_path.starts_with(crit) {
            return Err(RiskyPathReason::CriticalDirectory);
        }
    }

    Ok(())
}

/// # Unchecked Operation
/// This function does not check whether or not the given path is already
/// canonicalized. Please make sure to canonicalize the path beforehand.
fn check_windows_path_risk(canon_path: &Path) -> Result<(), RiskyPathReason> {
    if canon_path.starts_with(r"\\.\") {
        return Err(RiskyPathReason::CriticalDirectory);
    }

    let path_str = canon_path.to_string_lossy();
    let path_str = path_str.trim_start_matches(r"\\?\");

    if path_str.starts_with(r"\\") {
        // We don't know what UNC paths are "critical" for
        // the user
        return Err(RiskyPathReason::UnknownPath);
    }

    // Now path_str should be like C:\...
    // We check just in case
    let mut iter = path_str.chars();
    if iter.next().is_none_or(|char| !char.is_ascii_alphabetic()) {
        return Err(RiskyPathReason::UnknownPath);
    }
    if iter.next().is_none_or(|char| char != ':') {
        return Err(RiskyPathReason::UnknownPath);
    }
    if iter.next().is_none_or(|char| char != '\\') {
        return Err(RiskyPathReason::UnknownPath);
    }

    // The path after `X:\`, e.g. `windows` or `system volume information`.
    // This is lowercase!
    let inner_path_lower = path_str[3..].to_lowercase();

    // These dirs are critical, but not their contents
    let critical_exact_dirs = ["users"];

    // These dirs are critical along with their contents
    let critical_prefix_dirs = [
        // Implicitly also `Windows.old`
        "windows",
        "system volume information",
        "$recycle.bin",
        "program files",
        "program files (x86)",
        "programdata",
        "recovery",
        "perflogs",
    ];

    for crit in critical_exact_dirs {
        if &*inner_path_lower == crit {
            return Err(RiskyPathReason::CriticalDirectory);
        }
    }

    for crit in critical_prefix_dirs {
        if inner_path_lower.starts_with(crit) {
            return Err(RiskyPathReason::CriticalDirectory);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_unix_safe_paths() {
        if cfg!(unix) {
            let safe_paths = [
                "/tmp",
                "/home/user/docs",
                "/mnt/data/somefolder",
                "/Users/foobar/docs",
            ];

            for path in safe_paths {
                let result = check_unix_path_risk(Path::new(path));
                assert!(
                    result.is_ok(),
                    "Path `{path}` should be safe, got {result:?}"
                );
            }
        }
    }

    #[test]
    fn test_unix_risky_paths() {
        if cfg!(unix) {
            let critical_dirs = [
                ("/", RiskyPathReason::RootDirectory),
                ("/home", RiskyPathReason::CriticalDirectory),
                ("/usr/bin", RiskyPathReason::CriticalDirectory),
                ("/mnt", RiskyPathReason::CriticalDirectory),
                ("/Users", RiskyPathReason::CriticalDirectory), // Mac
                ("/system", RiskyPathReason::CriticalDirectory), // Android
            ];

            for (path, expected) in critical_dirs {
                let result = check_unix_path_risk(Path::new(path));
                let Err(reason) = result else {
                    panic!("Path `{path}` should be risky")
                };
                assert!(
                    core::mem::discriminant(&reason) == core::mem::discriminant(&expected),
                    "Path `{path}` should yield error {expected:?}, got {reason:?}",
                );
            }
        }
    }

    #[test]
    fn test_windows_safe_paths() {
        if cfg!(windows) {
            let safe_paths = [
                r"C:\Users\MyUser\Documents",
                r"C:\Temp",
                r"D:\Projects",
                r"\\?\D:\Projects",
            ];

            for path in safe_paths {
                let result = check_windows_path_risk(Path::new(path));
                assert!(
                    result.is_ok(),
                    "Path `{path}` should be safe, got {result:?}"
                );
            }
        }
    }

    #[test]
    fn test_windows_risky_paths() {
        if cfg!(windows) {
            let risky_paths = [
                (r"C:\", RiskyPathReason::RootDirectory),
                (r"\\?\C:\", RiskyPathReason::RootDirectory),
                (r"D:\", RiskyPathReason::RootDirectory),
                (r"\\?\D:\", RiskyPathReason::RootDirectory),
                (r"X:\", RiskyPathReason::RootDirectory),
                (r"\\?\X:\", RiskyPathReason::RootDirectory),
                (r"C:\Windows", RiskyPathReason::CriticalDirectory),
                (r"\\?\C:\Windows", RiskyPathReason::CriticalDirectory),
                (r"C:\Program Files", RiskyPathReason::CriticalDirectory),
                (r"\\?\C:\Program Files", RiskyPathReason::CriticalDirectory),
                (r"C:\Users", RiskyPathReason::CriticalDirectory),
                (r"\\Server\Share", RiskyPathReason::UnknownPath),
                (r"\\.\PhysicalDrive0", RiskyPathReason::CriticalDirectory),
            ];

            for (path, expected) in risky_paths {
                let result = check_windows_path_risk(Path::new(path));
                let Err(reason) = result else {
                    panic!("Path `{path}` should be risky")
                };
                assert!(
                    core::mem::discriminant(&reason) == core::mem::discriminant(&expected),
                    "Path `{path}` should yield error {expected:?}, got {reason:?}",
                );
            }
        }
    }
}
