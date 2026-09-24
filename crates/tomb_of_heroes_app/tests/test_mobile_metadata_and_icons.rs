//! Automated tests verifying mobile application naming, metadata, and icon assets
//! for both Android and iOS targets.

use std::error::Error;
use std::path::{Path, PathBuf};

fn get_workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/tomb_of_heroes_app -> root
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_png_dimensions(path: &Path) -> Result<(u32, u32), String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    if bytes.len() < 24 {
        return Err(format!("File too short: {}", path.display()));
    }
    if &bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(format!("File is not a valid PNG: {}", path.display()));
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    Ok((width, height))
}

#[test]
fn test_android_cargo_metadata_has_correct_name_and_icon() -> Result<(), Box<dyn Error>> {
    let root = get_workspace_root();
    let app_cargo_path = root.join("crates/tomb_of_heroes_app/Cargo.toml");
    let content = std::fs::read_to_string(&app_cargo_path)?;

    // 1. Verify cargo-apk resources folder definition
    assert!(
        content.contains("resources = \"res\""),
        "crates/tomb_of_heroes_app/Cargo.toml must define resources = \"res\" for cargo-apk"
    );

    // 2. Verify [package.metadata.android.application] section exists with label and icon
    assert!(
        content.contains("[package.metadata.android.application]"),
        "crates/tomb_of_heroes_app/Cargo.toml must have [package.metadata.android.application] table"
    );
    assert!(
        content.contains("icon = \"@mipmap/ic_launcher\""),
        "Application must reference icon = \"@mipmap/ic_launcher\""
    );

    // 3. Verify [package.metadata.android.application.activity] section has label
    assert!(
        content.contains("[package.metadata.android.application.activity]"),
        "crates/tomb_of_heroes_app/Cargo.toml must have [package.metadata.android.application.activity] table"
    );

    // 4. Ensure application label is 'Tomb of Heroes'
    let app_section = content
        .split("[package.metadata.android.application]")
        .nth(1)
        .unwrap_or_default();
    let app_section_content = app_section.split('[').next().unwrap_or_default();
    assert!(
        app_section_content.contains("label = \"Tomb of Heroes\""),
        "Application label must be 'Tomb of Heroes'"
    );

    // 5. Ensure activity label is 'Tomb of Heroes'
    let activity_section = content
        .split("[package.metadata.android.application.activity]")
        .nth(1)
        .unwrap_or_default();
    let activity_section_content = activity_section.split('[').next().unwrap_or_default();
    assert!(
        activity_section_content.contains("label = \"Tomb of Heroes\""),
        "Activity label must be 'Tomb of Heroes'"
    );

    Ok(())
}

#[test]
fn test_android_mipmap_icons_exist_with_valid_dimensions() -> Result<(), Box<dyn Error>> {
    let root = get_workspace_root();
    let res_dir = root.join("crates/tomb_of_heroes_app/res");

    let densities = [
        ("mipmap-mdpi", 48),
        ("mipmap-hdpi", 72),
        ("mipmap-xhdpi", 96),
        ("mipmap-xxhdpi", 144),
        ("mipmap-xxxhdpi", 192),
    ];

    for (folder, expected_size) in densities {
        let icon_path = res_dir.join(folder).join("ic_launcher.png");
        let round_path = res_dir.join(folder).join("ic_launcher_round.png");

        assert!(
            icon_path.exists(),
            "Missing Android launcher icon: {}",
            icon_path.display()
        );
        assert!(
            round_path.exists(),
            "Missing Android round icon: {}",
            round_path.display()
        );

        let (w, h) = read_png_dimensions(&icon_path)?;
        assert_eq!(
            (w, h),
            (expected_size, expected_size),
            "Icon {} has dimensions {}x{}, expected {}x{}",
            icon_path.display(),
            w,
            h,
            expected_size,
            expected_size
        );

        let (rw, rh) = read_png_dimensions(&round_path)?;
        assert_eq!(
            (rw, rh),
            (expected_size, expected_size),
            "Round icon {} has dimensions {}x{}, expected {}x{}",
            round_path.display(),
            rw,
            rh,
            expected_size,
            expected_size
        );
    }

    Ok(())
}

#[test]
fn test_ios_plist_has_correct_name_and_icon_declarations() -> Result<(), Box<dyn Error>> {
    let root = get_workspace_root();
    let plist_path = root.join("packaging/ios/Info.plist");
    let content = std::fs::read_to_string(&plist_path)?;

    // App display name & bundle name
    assert!(
        content.contains("<key>CFBundleDisplayName</key>\n    <string>Tomb of Heroes</string>"),
        "iOS CFBundleDisplayName must be 'Tomb of Heroes'"
    );
    assert!(
        content.contains("<key>CFBundleName</key>\n    <string>Tomb of Heroes</string>"),
        "iOS CFBundleName must be 'Tomb of Heroes'"
    );

    // Icon declarations
    assert!(
        content.contains("<key>CFBundleIcons</key>"),
        "Info.plist must declare CFBundleIcons"
    );
    assert!(
        content.contains("<key>CFBundleIcons~ipad</key>"),
        "Info.plist must declare CFBundleIcons~ipad"
    );
    assert!(
        content.contains("<key>CFBundleIconFiles</key>"),
        "Info.plist must declare CFBundleIconFiles"
    );

    Ok(())
}

#[test]
fn test_ios_icon_files_exist_with_valid_dimensions() -> Result<(), Box<dyn Error>> {
    let root = get_workspace_root();
    let icons_dir = root.join("packaging/ios/icons");

    let required_icons = [
        ("AppIcon20x20@2x.png", 40),
        ("AppIcon20x20@3x.png", 60),
        ("AppIcon29x29@2x.png", 58),
        ("AppIcon29x29@3x.png", 87),
        ("AppIcon40x40@2x.png", 80),
        ("AppIcon40x40@3x.png", 120),
        ("AppIcon60x60@2x.png", 120),
        ("AppIcon60x60@3x.png", 180),
        ("AppIcon76x76@2x~ipad.png", 152),
        ("AppIcon83.5x83.5@2x~ipad.png", 167),
        ("AppIcon1024x1024.png", 1024),
    ];

    for (filename, expected_size) in required_icons {
        let file_path = icons_dir.join(filename);
        assert!(
            file_path.exists(),
            "Missing iOS icon file: {}",
            file_path.display()
        );

        let (w, h) = read_png_dimensions(&file_path)?;
        assert_eq!(
            (w, h),
            (expected_size, expected_size),
            "Icon {} has dimensions {}x{}, expected {}x{}",
            file_path.display(),
            w,
            h,
            expected_size,
            expected_size
        );
    }

    Ok(())
}
