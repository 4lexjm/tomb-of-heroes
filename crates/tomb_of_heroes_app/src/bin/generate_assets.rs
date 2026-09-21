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

    let logo_src = Path::new(
        "/home/runner/.gemini/antigravity-cli/brain/445ff1b9-530b-4516-8ac9-d4bec8d1b3cb/game_logo_macabre_1790010198229.jpg",
    );
    if logo_src.exists() {
        if let Ok(img) = image::open(logo_src) {
            let _ = fs::create_dir_all("assets/branding");
            let _ = fs::create_dir_all("assets/textures");

            // Save full logo PNG
            let logo_png = Path::new("assets/branding/logo.png");
            let texture_logo = Path::new("assets/textures/game_logo.png");
            if let Err(e) = img.save_with_format(logo_png, image::ImageFormat::Png) {
                eprintln!("Failed to save logo.png: {e}");
            } else {
                let _ = img.save_with_format(texture_logo, image::ImageFormat::Png);
                println!("Successfully generated assets/branding/logo.png and assets/textures/game_logo.png");
            }

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
        }
    }
}
