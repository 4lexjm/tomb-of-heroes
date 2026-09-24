//! Application version and build metadata.
//!
//! Exposes compile-time semantic versioning, build number, and composite
//! version identifiers provided by Cargo and `build.rs`.

/// Base semantic version from `Cargo.toml` (e.g. `"0.1.0"`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build number (e.g. `"42"` from CI `GITHUB_RUN_NUMBER`, or `"dev"` for local developer builds).
pub const BUILD_NUMBER: &str = env!("APP_BUILD_NUMBER");

/// Full composite SemVer 2.0 version string (e.g. `"0.1.0+build.42.d819447"` or `"0.1.0-dev"`).
pub const FULL_VERSION: &str = env!("APP_FULL_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants_are_non_empty() {
        assert!(!VERSION.is_empty(), "VERSION must not be empty");
        assert!(!BUILD_NUMBER.is_empty(), "BUILD_NUMBER must not be empty");
        assert!(!FULL_VERSION.is_empty(), "FULL_VERSION must not be empty");
        assert!(
            FULL_VERSION.starts_with(VERSION),
            "FULL_VERSION ('{FULL_VERSION}') must start with base VERSION ('{VERSION}')"
        );
    }

    #[test]
    fn test_version_components_structure() {
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert_eq!(
            parts.len(),
            3,
            "Base version must follow SemVer X.Y.Z structure"
        );
        // During conception phase, major version MUST remain 0
        assert_eq!(
            parts[0], "0",
            "Major version must strictly remain 0 during conception"
        );
    }
}
