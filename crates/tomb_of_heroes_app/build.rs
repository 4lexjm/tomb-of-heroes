//! Build script for `tomb_of_heroes_app`.
//!
//! Injects build number, optional commit SHA, and composite SemVer 2.0 version
//! strings into the compilation environment without unwrap or panic.

fn main() {
    println!("cargo:rerun-if-env-changed=APP_BUILD_NUMBER");
    println!("cargo:rerun-if-env-changed=APP_BUILD_SHA");

    let pkg_version = env!("CARGO_PKG_VERSION");
    let build_number = std::env::var("APP_BUILD_NUMBER").unwrap_or_else(|_| "dev".to_string());

    let sha_opt = std::env::var("APP_BUILD_SHA")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            let trimmed = s.trim();
            if trimmed.len() >= 7 {
                trimmed[..7].to_string()
            } else {
                trimmed.to_string()
            }
        })
        .or_else(|| {
            std::process::Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        });

    let full_version = match (build_number.as_str(), sha_opt) {
        ("dev", Some(sha)) => format!("{pkg_version}-dev+{sha}"),
        ("dev", None) => format!("{pkg_version}-dev"),
        (num, Some(sha)) => format!("{pkg_version}+build.{num}.{sha}"),
        (num, None) => format!("{pkg_version}+build.{num}"),
    };

    println!("cargo:rustc-env=APP_BUILD_NUMBER={build_number}");
    println!("cargo:rustc-env=APP_FULL_VERSION={full_version}");
}
