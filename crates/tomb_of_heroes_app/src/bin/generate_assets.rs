//! Standalone tool generating `assets/textures/dungeon_sheet.png`.

use std::path::Path;
use tomb_of_heroes_app::asset_gen::ensure_dungeon_sheet_exists;

fn main() {
    let path = Path::new("assets/textures/dungeon_sheet.png");
    if let Err(e) = ensure_dungeon_sheet_exists(path) {
        eprintln!("Failed to generate dungeon sheet: {e}");
        std::process::exit(1);
    }
    println!("Successfully generated assets/textures/dungeon_sheet.png");
}
