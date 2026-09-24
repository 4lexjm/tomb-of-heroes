//! Standalone tool generating `assets/textures/dungeon_sheet.png` and logo assets.

use std::fs;
use std::path::Path;
use tomb_of_heroes_app::asset_gen::ensure_dungeon_sheet_exists;

fn main() {
    let sheet_path = Path::new("assets/textures/dungeon_sheet.png");
    if let Err(e) = ensure_dungeon_sheet_exists(sheet_path) {
        eprintln!("Failed to generate dungeon sheet: {e}");
        std::process::exit(1);
    }
    println!("Successfully generated assets/textures/dungeon_sheet.png");

    let logo_primary = Path::new("assets/branding/logo.png");
    let logo_fallback = Path::new(
        "/home/runner/.gemini/antigravity-cli/brain/445ff1b9-530b-4516-8ac9-d4bec8d1b3cb/game_logo_macabre_1790010198229.jpg",
    );
    let logo_source = if logo_primary.exists() {
        Some(logo_primary)
    } else if logo_fallback.exists() {
        Some(logo_fallback)
    } else {
        None
    };

    if let Some(src) = logo_source {
        if let Ok(img) = image::open(src) {
            let _ = fs::create_dir_all("assets/branding");
            let _ = fs::create_dir_all("assets/textures");

            // Save full logo PNG if not already saved from this file
            let logo_png = Path::new("assets/branding/logo.png");
            let texture_logo = Path::new("assets/textures/game_logo.png");
            if src != logo_png {
                if let Err(e) = img.save_with_format(logo_png, image::ImageFormat::Png) {
                    eprintln!("Failed to save logo.png: {e}");
                }
            }
            let _ = img.save_with_format(texture_logo, image::ImageFormat::Png);
            println!(
                "Successfully ensured assets/branding/logo.png and assets/textures/game_logo.png"
            );

            // Save icons at multiple resolutions
            let icon_256 = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
            let _ =
                icon_256.save_with_format("assets/branding/icon_256.png", image::ImageFormat::Png);

            let icon_128 = img.resize(128, 128, image::imageops::FilterType::Lanczos3);
            let _ =
                icon_128.save_with_format("assets/branding/icon_128.png", image::ImageFormat::Png);

            let icon_64 = img.resize(64, 64, image::imageops::FilterType::Lanczos3);
            let _ =
                icon_64.save_with_format("assets/branding/icon_64.png", image::ImageFormat::Png);

            let icon_32 = img.resize(32, 32, image::imageops::FilterType::Lanczos3);
            let _ =
                icon_32.save_with_format("assets/branding/icon_32.png", image::ImageFormat::Png);

            println!("Successfully generated app icons (256, 128, 64, 32) in assets/branding/");

            // Generate Android mipmap launcher icons
            let mipmap_targets = [
                ("mipmap-mdpi", 48),
                ("mipmap-hdpi", 72),
                ("mipmap-xhdpi", 96),
                ("mipmap-xxhdpi", 144),
                ("mipmap-xxxhdpi", 192),
            ];

            let res_bases = ["crates/tomb_of_heroes_app/res", "res"];
            for base in res_bases {
                for (folder, size) in mipmap_targets {
                    let dir_path = format!("{base}/{folder}");
                    let _ = fs::create_dir_all(&dir_path);
                    let resized = img.resize(size, size, image::imageops::FilterType::Lanczos3);
                    let icon_path = format!("{dir_path}/ic_launcher.png");
                    let _ = resized.save_with_format(&icon_path, image::ImageFormat::Png);
                    let round_path = format!("{dir_path}/ic_launcher_round.png");
                    let _ = resized.save_with_format(&round_path, image::ImageFormat::Png);
                }
            }
            println!("Successfully generated Android mipmap launcher icons in res/");

            // Generate iOS app icons
            let ios_icon_targets = [
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
                ("Icon-60@2x.png", 120),
                ("Icon-60@3x.png", 180),
                ("Icon-76.png", 76),
                ("Icon-76@2x.png", 152),
                ("Icon-83.5@2x.png", 167),
                ("Icon-1024.png", 1024),
            ];

            let ios_dir = "packaging/ios/icons";
            let _ = fs::create_dir_all(ios_dir);
            for (filename, size) in ios_icon_targets {
                let icon_path = format!("{ios_dir}/{filename}");
                let resized = img.resize(size, size, image::imageops::FilterType::Lanczos3);
                let _ = resized.save_with_format(&icon_path, image::ImageFormat::Png);
            }
            println!("Successfully generated iOS icons in packaging/ios/icons/");
        }
    }
}
