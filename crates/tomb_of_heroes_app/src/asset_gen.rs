//! Procedural 16x16 Pixel-Art Asset Pipeline for *Tomb of Heroes*.
//!
//! Synthesizes the master sprite sheet `assets/textures/dungeon_sheet.png`
//! using the retro DB16 palette.
//!
//! Conforms to `SPEC-REQ-FRONT-002`.

use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;

use image::{ImageFormat, Rgba, RgbaImage};

/// Size of an individual sprite tile in pixels.
pub const TILE_SIZE: u32 = 16;
/// Number of sprite columns in the master sheet.
pub const SHEET_COLS: u32 = 8;
/// Number of sprite rows in the master sheet.
pub const SHEET_ROWS: u32 = 4;
/// Total sheet width in pixels (8 * 16 = 128).
pub const SHEET_WIDTH: u32 = SHEET_COLS * TILE_SIZE;
/// Total sheet height in pixels (4 * 16 = 64).
pub const SHEET_HEIGHT: u32 = SHEET_ROWS * TILE_SIZE;

// ─────────────────────────────────────────────────────────────────────────────
// Palette DB16 (DawnBringer 16)
// ─────────────────────────────────────────────────────────────────────────────
pub const COLOR_VOID: Rgba<u8> = Rgba([0x14, 0x10, 0x13, 0xFF]);
pub const COLOR_DARK_PURPLE: Rgba<u8> = Rgba([0x3B, 0x17, 0x25, 0xFF]);
pub const COLOR_NIGHT_BLUE: Rgba<u8> = Rgba([0x1A, 0x1C, 0x2C, 0xFF]);
pub const COLOR_SLATE_GRAY: Rgba<u8> = Rgba([0x3B, 0x3A, 0x4A, 0xFF]);
pub const COLOR_STONE_GRAY: Rgba<u8> = Rgba([0x56, 0x6C, 0x86, 0xFF]);
pub const COLOR_POISON_GREEN: Rgba<u8> = Rgba([0x33, 0x98, 0x4B, 0xFF]);
pub const COLOR_MOSS_GREEN: Rgba<u8> = Rgba([0x5A, 0xC5, 0x4F, 0xFF]);
pub const COLOR_LEATHER_BROWN: Rgba<u8> = Rgba([0x73, 0x3E, 0x39, 0xFF]);
pub const COLOR_SAND_ORANGE: Rgba<u8> = Rgba([0xCF, 0x65, 0x1F, 0xFF]);
pub const COLOR_ANCIENT_GOLD: Rgba<u8> = Rgba([0xF4, 0xB4, 0x1B, 0xFF]);
pub const COLOR_BONE_WHITE: Rgba<u8> = Rgba([0xF0, 0xD6, 0xB6, 0xFF]);
pub const COLOR_METAL_GRAY: Rgba<u8> = Rgba([0x94, 0xB0, 0xC2, 0xFF]);
pub const COLOR_MAGIC_BLUE: Rgba<u8> = Rgba([0x41, 0x7E, 0xBD, 0xFF]);
pub const COLOR_BLOOD_RED: Rgba<u8> = Rgba([0x8A, 0x19, 0x23, 0xFF]);
pub const COLOR_CRIMSON_RED: Rgba<u8> = Rgba([0xA5, 0x30, 0x30, 0xFF]);
pub const COLOR_PURE_WHITE: Rgba<u8> = Rgba([0xFF, 0xFF, 0xFF, 0xFF]);
pub const COLOR_TRANSPARENT: Rgba<u8> = Rgba([0x00, 0x00, 0x00, 0x00]);

/// Type-safe numeric sprite indices in the atlas layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(usize)]
pub enum SpriteIndex {
    FloorTile = 0,
    WallTile = 1,
    StairsDown = 2,
    StairsUp = 3,
    AcidPit = 4,
    PortcullisClosed = 5,
    PortcullisOpen = 6,
    SanctifiedFloor = 7,
    HeroWarrior = 8,
    HeroCleric = 9,
    HeroPaladin = 10,
    HeroMage = 11,
    HeroRogue = 12,
    HeroNecromancer = 13,
    HeroPanicked = 14,
    HeroRetreating = 15,
    MonsterSkeleton = 16,
    MonsterZombie = 17,
    MonsterSpectre = 18,
    BossOverlord = 19,
    TrapSpikes = 20,
    TrapSprung = 21,
    SoulEssence = 22,
    ExplosionFx = 23,
    CorpseIntact = 24,
    CorpseDamaged = 25,
    CorpseBones = 26,
    FogUnexplored = 27,
    FogExplored = 28,
    IntelCompromised = 29,
    TerrorMiasma = 30,
    TileCursor = 31,
}

impl SpriteIndex {
    /// Returns the integer atlas index.
    #[inline]
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Sets a pixel at local coordinates within a specific sprite tile.
fn set_px(img: &mut RgbaImage, col: u32, row: u32, lx: u32, ly: u32, color: Rgba<u8>) {
    if lx < TILE_SIZE && ly < TILE_SIZE && col < SHEET_COLS && row < SHEET_ROWS {
        let gx = col * TILE_SIZE + lx;
        let gy = row * TILE_SIZE + ly;
        img.put_pixel(gx, gy, color);
    }
}

/// Fills an entire 16x16 sprite tile with a base color.
fn fill_tile(img: &mut RgbaImage, col: u32, row: u32, color: Rgba<u8>) {
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            set_px(img, col, row, x, y, color);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tile Drawers (Row 0)
// ─────────────────────────────────────────────────────────────────────────────

fn draw_floor_tile(img: &mut RgbaImage, col: u32, row: u32) {
    fill_tile(img, col, row, COLOR_STONE_GRAY);
    // Grout borders and cobblestone patterns
    for i in 0..16 {
        set_px(img, col, row, i, 0, COLOR_SLATE_GRAY);
        set_px(img, col, row, 0, i, COLOR_SLATE_GRAY);
        set_px(img, col, row, i, 15, COLOR_NIGHT_BLUE);
        set_px(img, col, row, 15, i, COLOR_NIGHT_BLUE);
    }
    // Mid grooves
    for x in 1..15 {
        set_px(img, col, row, x, 8, COLOR_SLATE_GRAY);
    }
    for y in 1..8 {
        set_px(img, col, row, 7, y, COLOR_SLATE_GRAY);
    }
    for y in 9..15 {
        set_px(img, col, row, 11, y, COLOR_SLATE_GRAY);
    }
    // Subtle surface texture dots
    set_px(img, col, row, 3, 3, COLOR_METAL_GRAY);
    set_px(img, col, row, 11, 4, COLOR_METAL_GRAY);
    set_px(img, col, row, 4, 12, COLOR_METAL_GRAY);
}

fn draw_wall_tile(img: &mut RgbaImage, col: u32, row: u32) {
    fill_tile(img, col, row, COLOR_NIGHT_BLUE);
    // Brick rows
    for y in [1, 2, 3, 4, 6, 7, 8, 9, 11, 12, 13, 14] {
        for x in 1..15 {
            set_px(img, col, row, x, y, COLOR_SLATE_GRAY);
        }
    }
    // Mortar lines
    for x in 0..16 {
        set_px(img, col, row, x, 0, COLOR_VOID);
        set_px(img, col, row, x, 5, COLOR_VOID);
        set_px(img, col, row, x, 10, COLOR_VOID);
        set_px(img, col, row, x, 15, COLOR_VOID);
    }
    // Vertical joints
    for y in 1..5 {
        set_px(img, col, row, 7, y, COLOR_VOID);
    }
    for y in 6..10 {
        set_px(img, col, row, 3, y, COLOR_VOID);
        set_px(img, col, row, 11, y, COLOR_VOID);
    }
    for y in 11..15 {
        set_px(img, col, row, 7, y, COLOR_VOID);
    }
    // Top brick highlight
    for x in 1..7 {
        set_px(img, col, row, x, 1, COLOR_STONE_GRAY);
    }
    for x in 8..15 {
        set_px(img, col, row, x, 1, COLOR_STONE_GRAY);
    }
}

fn draw_stairs_down(img: &mut RgbaImage, col: u32, row: u32) {
    fill_tile(img, col, row, COLOR_STONE_GRAY);
    // Steps going down into darkness
    let steps = [
        (1, 3, COLOR_METAL_GRAY),
        (4, 6, COLOR_STONE_GRAY),
        (7, 9, COLOR_SLATE_GRAY),
        (10, 12, COLOR_NIGHT_BLUE),
        (13, 15, COLOR_VOID),
    ];
    for (y_start, y_end, color) in steps {
        for y in y_start..=y_end {
            for x in 1..15 {
                set_px(img, col, row, x, y, color);
            }
        }
        for x in 1..15 {
            set_px(img, col, row, x, y_start, COLOR_PURE_WHITE);
        }
    }
    // Border
    for i in 0..16 {
        set_px(img, col, row, i, 0, COLOR_SLATE_GRAY);
        set_px(img, col, row, 0, i, COLOR_SLATE_GRAY);
        set_px(img, col, row, 15, i, COLOR_SLATE_GRAY);
    }
}

fn draw_stairs_up(img: &mut RgbaImage, col: u32, row: u32) {
    fill_tile(img, col, row, COLOR_SLATE_GRAY);
    // Steps ascending toward light
    let steps = [
        (1, 3, COLOR_VOID),
        (4, 6, COLOR_NIGHT_BLUE),
        (7, 9, COLOR_SLATE_GRAY),
        (10, 12, COLOR_STONE_GRAY),
        (13, 15, COLOR_ANCIENT_GOLD),
    ];
    for (y_start, y_end, color) in steps {
        for y in y_start..=y_end {
            for x in 1..15 {
                set_px(img, col, row, x, y, color);
            }
        }
        for x in 1..15 {
            set_px(img, col, row, x, y_end, COLOR_PURE_WHITE);
        }
    }
}

fn draw_acid_pit(img: &mut RgbaImage, col: u32, row: u32) {
    fill_tile(img, col, row, COLOR_POISON_GREEN);
    // Pit rim
    for i in 0..16 {
        set_px(img, col, row, i, 0, COLOR_SLATE_GRAY);
        set_px(img, col, row, i, 15, COLOR_SLATE_GRAY);
        set_px(img, col, row, 0, i, COLOR_SLATE_GRAY);
        set_px(img, col, row, 15, i, COLOR_SLATE_GRAY);
    }
    // Acid depth & bubbles
    for y in 2..14 {
        for x in 2..14 {
            set_px(img, col, row, x, y, COLOR_POISON_GREEN);
        }
    }
    // Toxic bubbling highlights
    set_px(img, col, row, 5, 5, COLOR_MOSS_GREEN);
    set_px(img, col, row, 6, 5, COLOR_PURE_WHITE);
    set_px(img, col, row, 5, 6, COLOR_MOSS_GREEN);
    set_px(img, col, row, 10, 9, COLOR_MOSS_GREEN);
    set_px(img, col, row, 11, 9, COLOR_PURE_WHITE);
    set_px(img, col, row, 9, 10, COLOR_MOSS_GREEN);
    set_px(img, col, row, 3, 11, COLOR_MOSS_GREEN);
}

fn draw_portcullis(img: &mut RgbaImage, col: u32, row: u32, is_open: bool) {
    fill_tile(img, col, row, COLOR_STONE_GRAY);
    // Stone arch frame
    for y in 0..16 {
        set_px(img, col, row, 1, y, COLOR_SLATE_GRAY);
        set_px(img, col, row, 2, y, COLOR_NIGHT_BLUE);
        set_px(img, col, row, 13, y, COLOR_NIGHT_BLUE);
        set_px(img, col, row, 14, y, COLOR_SLATE_GRAY);
    }
    for x in 1..15 {
        set_px(img, col, row, x, 1, COLOR_SLATE_GRAY);
        set_px(img, col, row, x, 2, COLOR_NIGHT_BLUE);
    }
    let bar_bottom = if is_open { 6 } else { 15 };
    // Iron vertical bars
    for x in [4, 7, 10] {
        for y in 2..bar_bottom {
            set_px(img, col, row, x, y, COLOR_METAL_GRAY);
        }
        // Spike bottom
        set_px(img, col, row, x, bar_bottom, COLOR_PURE_WHITE);
    }
    // Horizontal crossbars
    if !is_open {
        for x in 3..13 {
            set_px(img, col, row, x, 6, COLOR_SLATE_GRAY);
            set_px(img, col, row, x, 11, COLOR_SLATE_GRAY);
        }
        // Golden lock mechanism in center
        set_px(img, col, row, 7, 8, COLOR_ANCIENT_GOLD);
        set_px(img, col, row, 8, 8, COLOR_ANCIENT_GOLD);
    }
}

fn draw_sanctified_floor(img: &mut RgbaImage, col: u32, row: u32) {
    draw_floor_tile(img, col, row);
    // Holy radiant cross emblem
    for y in 3..13 {
        set_px(img, col, row, 7, y, COLOR_ANCIENT_GOLD);
        set_px(img, col, row, 8, y, COLOR_ANCIENT_GOLD);
    }
    for x in 4..12 {
        set_px(img, col, row, x, 6, COLOR_ANCIENT_GOLD);
        set_px(img, col, row, x, 7, COLOR_ANCIENT_GOLD);
    }
    // Holy core glow
    set_px(img, col, row, 7, 6, COLOR_PURE_WHITE);
    set_px(img, col, row, 8, 6, COLOR_PURE_WHITE);
    set_px(img, col, row, 7, 7, COLOR_PURE_WHITE);
    set_px(img, col, row, 8, 7, COLOR_PURE_WHITE);
}

// ─────────────────────────────────────────────────────────────────────────────
// Hero Drawers (Row 1)
// ─────────────────────────────────────────────────────────────────────────────

fn draw_warrior(img: &mut RgbaImage, col: u32, row: u32) {
    // Silver Helmet
    for y in 2..5 {
        for x in 6..10 {
            set_px(img, col, row, x, y, COLOR_METAL_GRAY);
        }
    }
    // Visor slit
    set_px(img, col, row, 7, 4, COLOR_VOID);
    set_px(img, col, row, 8, 4, COLOR_VOID);
    // Armor torso (blue tunic over chainmail)
    for y in 5..10 {
        for x in 5..11 {
            set_px(img, col, row, x, y, COLOR_MAGIC_BLUE);
        }
    }
    // Sword in right hand
    for y in 5..12 {
        set_px(img, col, row, 12, y, COLOR_PURE_WHITE);
    }
    set_px(img, col, row, 11, 7, COLOR_ANCIENT_GOLD);
    set_px(img, col, row, 13, 7, COLOR_ANCIENT_GOLD);
    // Shield on left arm
    for y in 6..11 {
        for x in 2..5 {
            set_px(img, col, row, x, y, COLOR_METAL_GRAY);
        }
    }
    set_px(img, col, row, 3, 8, COLOR_ANCIENT_GOLD);
    // Legs & boots
    for y in 10..14 {
        set_px(img, col, row, 6, y, COLOR_LEATHER_BROWN);
        set_px(img, col, row, 9, y, COLOR_LEATHER_BROWN);
    }
}

fn draw_cleric(img: &mut RgbaImage, col: u32, row: u32) {
    // White cowl
    for y in 2..5 {
        for x in 6..10 {
            set_px(img, col, row, x, y, COLOR_BONE_WHITE);
        }
    }
    // Face
    set_px(img, col, row, 7, 4, COLOR_SAND_ORANGE);
    set_px(img, col, row, 8, 4, COLOR_SAND_ORANGE);
    // Robe
    for y in 5..12 {
        for x in 5..11 {
            set_px(img, col, row, x, y, COLOR_BONE_WHITE);
        }
    }
    // Gold sash and holy cross
    for x in 6..10 {
        set_px(img, col, row, x, 7, COLOR_ANCIENT_GOLD);
    }
    set_px(img, col, row, 7, 8, COLOR_ANCIENT_GOLD);
    set_px(img, col, row, 8, 8, COLOR_ANCIENT_GOLD);
    // Holy mace / hammer
    for y in 4..12 {
        set_px(img, col, row, 12, y, COLOR_LEATHER_BROWN);
    }
    for x in 11..14 {
        set_px(img, col, row, x, 4, COLOR_ANCIENT_GOLD);
        set_px(img, col, row, x, 5, COLOR_ANCIENT_GOLD);
    }
}

fn draw_paladin(img: &mut RgbaImage, col: u32, row: u32) {
    // Golden ornate helmet
    for y in 2..5 {
        for x in 6..10 {
            set_px(img, col, row, x, y, COLOR_ANCIENT_GOLD);
        }
    }
    set_px(img, col, row, 7, 4, COLOR_MAGIC_BLUE);
    set_px(img, col, row, 8, 4, COLOR_MAGIC_BLUE);
    // Golden plate armor with royal red cape
    for y in 5..10 {
        for x in 4..12 {
            set_px(img, col, row, x, y, COLOR_CRIMSON_RED);
        }
        for x in 6..10 {
            set_px(img, col, row, x, y, COLOR_ANCIENT_GOLD);
        }
    }
    // Tower shield with glowing cross
    for y in 5..12 {
        for x in 2..5 {
            set_px(img, col, row, x, y, COLOR_ANCIENT_GOLD);
        }
    }
    set_px(img, col, row, 3, 7, COLOR_PURE_WHITE);
    set_px(img, col, row, 3, 8, COLOR_PURE_WHITE);
    set_px(img, col, row, 3, 9, COLOR_PURE_WHITE);
    // Golden warhammer
    for y in 4..12 {
        set_px(img, col, row, 12, y, COLOR_LEATHER_BROWN);
    }
    for x in 11..14 {
        set_px(img, col, row, x, 3, COLOR_METAL_GRAY);
        set_px(img, col, row, x, 4, COLOR_METAL_GRAY);
    }
}

fn draw_mage(img: &mut RgbaImage, col: u32, row: u32) {
    // Pointed Wizard Hat
    set_px(img, col, row, 8, 1, COLOR_DARK_PURPLE);
    set_px(img, col, row, 8, 2, COLOR_DARK_PURPLE);
    for x in 7..10 {
        set_px(img, col, row, x, 3, COLOR_DARK_PURPLE);
    }
    for x in 5..12 {
        set_px(img, col, row, x, 4, COLOR_DARK_PURPLE);
    }
    // Face & beard
    set_px(img, col, row, 7, 5, COLOR_SAND_ORANGE);
    set_px(img, col, row, 8, 5, COLOR_SAND_ORANGE);
    set_px(img, col, row, 7, 6, COLOR_PURE_WHITE);
    set_px(img, col, row, 8, 6, COLOR_PURE_WHITE);
    // Blue Mystic Robe
    for y in 6..13 {
        for x in 5..11 {
            set_px(img, col, row, x, y, COLOR_MAGIC_BLUE);
        }
    }
    // Glowing Arcane Staff
    for y in 3..14 {
        set_px(img, col, row, 12, y, COLOR_LEATHER_BROWN);
    }
    // Staff crystal orb
    set_px(img, col, row, 12, 2, COLOR_PURE_WHITE);
    set_px(img, col, row, 11, 2, COLOR_MOSS_GREEN);
    set_px(img, col, row, 13, 2, COLOR_MOSS_GREEN);
}

fn draw_rogue(img: &mut RgbaImage, col: u32, row: u32) {
    // Dark green hood
    for y in 2..5 {
        for x in 6..10 {
            set_px(img, col, row, x, y, COLOR_POISON_GREEN);
        }
    }
    // Glowing eyes in shadow
    set_px(img, col, row, 6, 4, COLOR_VOID);
    set_px(img, col, row, 7, 4, COLOR_MOSS_GREEN);
    set_px(img, col, row, 8, 4, COLOR_MOSS_GREEN);
    set_px(img, col, row, 9, 4, COLOR_VOID);
    // Leather cloak and jerkin
    for y in 5..10 {
        for x in 5..11 {
            set_px(img, col, row, x, y, COLOR_LEATHER_BROWN);
        }
    }
    // Dual daggers
    for y in 6..10 {
        set_px(img, col, row, 3, y, COLOR_METAL_GRAY);
        set_px(img, col, row, 12, y, COLOR_METAL_GRAY);
    }
    set_px(img, col, row, 3, 5, COLOR_PURE_WHITE);
    set_px(img, col, row, 12, 5, COLOR_PURE_WHITE);
    // Agile boots
    for y in 10..14 {
        set_px(img, col, row, 6, y, COLOR_SLATE_GRAY);
        set_px(img, col, row, 9, y, COLOR_SLATE_GRAY);
    }
}

fn draw_necromancer(img: &mut RgbaImage, col: u32, row: u32) {
    // Dark Purple Cowl
    for y in 2..5 {
        for x in 6..10 {
            set_px(img, col, row, x, y, COLOR_DARK_PURPLE);
        }
    }
    // Pale gaunt face
    set_px(img, col, row, 7, 4, COLOR_BONE_WHITE);
    set_px(img, col, row, 8, 4, COLOR_BONE_WHITE);
    // Ragged dark robes
    for y in 5..13 {
        for x in 5..11 {
            set_px(img, col, row, x, y, COLOR_VOID);
        }
    }
    // Skull Staff
    for y in 4..14 {
        set_px(img, col, row, 12, y, COLOR_BONE_WHITE);
    }
    set_px(img, col, row, 12, 2, COLOR_BONE_WHITE);
    set_px(img, col, row, 11, 3, COLOR_BONE_WHITE);
    set_px(img, col, row, 13, 3, COLOR_BONE_WHITE);
    // Red glowing eye in skull
    set_px(img, col, row, 12, 3, COLOR_BLOOD_RED);
}

fn draw_hero_panicked(img: &mut RgbaImage, col: u32, row: u32) {
    draw_warrior(img, col, row);
    // Flailing arms / sweat drops
    set_px(img, col, row, 3, 2, COLOR_MAGIC_BLUE);
    set_px(img, col, row, 12, 2, COLOR_MAGIC_BLUE);
    // Wide fearful eyes
    set_px(img, col, row, 6, 4, COLOR_PURE_WHITE);
    set_px(img, col, row, 9, 4, COLOR_PURE_WHITE);
}

fn draw_hero_retreating(img: &mut RgbaImage, col: u32, row: u32) {
    draw_rogue(img, col, row);
    // Dust trail behind running feet
    set_px(img, col, row, 1, 13, COLOR_BONE_WHITE);
    set_px(img, col, row, 2, 12, COLOR_BONE_WHITE);
    set_px(img, col, row, 3, 14, COLOR_BONE_WHITE);
}

// ─────────────────────────────────────────────────────────────────────────────
// Monster Drawers (Row 2)
// ─────────────────────────────────────────────────────────────────────────────

fn draw_skeleton(img: &mut RgbaImage, col: u32, row: u32) {
    // Skull
    for y in 2..5 {
        for x in 6..10 {
            set_px(img, col, row, x, y, COLOR_BONE_WHITE);
        }
    }
    // Glowing red eye sockets
    set_px(img, col, row, 7, 4, COLOR_BLOOD_RED);
    set_px(img, col, row, 8, 4, COLOR_BLOOD_RED);
    // Ribcage
    for y in 5..9 {
        set_px(img, col, row, 7, y, COLOR_BONE_WHITE);
        set_px(img, col, row, 8, y, COLOR_BONE_WHITE);
    }
    for x in 5..11 {
        set_px(img, col, row, x, 6, COLOR_BONE_WHITE);
        set_px(img, col, row, x, 8, COLOR_BONE_WHITE);
    }
    // Scimitar in hand
    for y in 4..12 {
        set_px(img, col, row, 12, y, COLOR_METAL_GRAY);
    }
    set_px(img, col, row, 11, 4, COLOR_METAL_GRAY);
    // Bone legs
    for y in 9..14 {
        set_px(img, col, row, 6, y, COLOR_BONE_WHITE);
        set_px(img, col, row, 9, y, COLOR_BONE_WHITE);
    }
}

fn draw_zombie(img: &mut RgbaImage, col: u32, row: u32) {
    // Decaying green-grey head
    for y in 2..5 {
        for x in 6..10 {
            set_px(img, col, row, x, y, COLOR_MOSS_GREEN);
        }
    }
    // Rotting hollow eye
    set_px(img, col, row, 7, 3, COLOR_VOID);
    set_px(img, col, row, 8, 4, COLOR_PURE_WHITE);
    // Tattered clothes & rotting torso
    for y in 5..10 {
        for x in 5..11 {
            set_px(img, col, row, x, y, COLOR_LEATHER_BROWN);
        }
    }
    // Exposed ribs / flesh wound
    set_px(img, col, row, 7, 7, COLOR_BLOOD_RED);
    set_px(img, col, row, 8, 7, COLOR_BLOOD_RED);
    // Outstretched grasping arms
    for x in 2..6 {
        set_px(img, col, row, x, 6, COLOR_MOSS_GREEN);
    }
    for x in 10..14 {
        set_px(img, col, row, x, 6, COLOR_MOSS_GREEN);
    }
    // Shambling legs
    for y in 10..14 {
        set_px(img, col, row, 6, y, COLOR_SLATE_GRAY);
        set_px(img, col, row, 9, y, COLOR_SLATE_GRAY);
    }
}

fn draw_spectre(img: &mut RgbaImage, col: u32, row: u32) {
    // Ethereal ghostly form
    for y in 2..10 {
        for x in 5..11 {
            set_px(img, col, row, x, y, COLOR_DARK_PURPLE);
        }
    }
    // Ghost face
    set_px(img, col, row, 7, 4, COLOR_MAGIC_BLUE);
    set_px(img, col, row, 8, 4, COLOR_MAGIC_BLUE);
    set_px(img, col, row, 7, 6, COLOR_VOID);
    set_px(img, col, row, 8, 6, COLOR_VOID);
    // Floating vapor wisps
    set_px(img, col, row, 5, 11, COLOR_DARK_PURPLE);
    set_px(img, col, row, 7, 12, COLOR_DARK_PURPLE);
    set_px(img, col, row, 9, 11, COLOR_DARK_PURPLE);
    set_px(img, col, row, 11, 13, COLOR_DARK_PURPLE);
}

fn draw_boss(img: &mut RgbaImage, col: u32, row: u32) {
    // Arch-Lich / Overlord: Horned Crown
    set_px(img, col, row, 4, 1, COLOR_ANCIENT_GOLD);
    set_px(img, col, row, 11, 1, COLOR_ANCIENT_GOLD);
    for x in 5..11 {
        set_px(img, col, row, x, 2, COLOR_ANCIENT_GOLD);
    }
    // Dark Skull Face with glowing purple eyes
    for y in 3..6 {
        for x in 6..10 {
            set_px(img, col, row, x, y, COLOR_VOID);
        }
    }
    set_px(img, col, row, 7, 4, COLOR_DARK_PURPLE);
    set_px(img, col, row, 8, 4, COLOR_DARK_PURPLE);
    // Towering crimson cape & dark armor
    for y in 6..14 {
        for x in 3..13 {
            set_px(img, col, row, x, y, COLOR_CRIMSON_RED);
        }
        for x in 6..10 {
            set_px(img, col, row, x, y, COLOR_VOID);
        }
    }
    // Golden pauldrons
    for x in 3..6 {
        set_px(img, col, row, x, 6, COLOR_ANCIENT_GOLD);
    }
    for x in 10..13 {
        set_px(img, col, row, x, 6, COLOR_ANCIENT_GOLD);
    }
    // Glowing Soul Gem in chest
    set_px(img, col, row, 7, 8, COLOR_PURE_WHITE);
    set_px(img, col, row, 8, 8, COLOR_PURE_WHITE);
}

fn draw_trap_spikes(img: &mut RgbaImage, col: u32, row: u32, sprung: bool) {
    draw_floor_tile(img, col, row);
    let spike_height = if sprung { 3 } else { 7 };
    // Sharp metal spikes rising from floor
    let coords: [(u32, u32); 5] = [(3, 12), (7, 10), (11, 12), (5, 6), (9, 6)];
    for &(sx, base_y) in &coords {
        for h in 0..spike_height {
            let y = base_y.saturating_sub(h);
            set_px(img, col, row, sx, y, COLOR_METAL_GRAY);
        }
        let tip_y = base_y.saturating_sub(spike_height);
        set_px(img, col, row, sx, tip_y, COLOR_PURE_WHITE);
        if sprung {
            set_px(img, col, row, sx, tip_y.saturating_add(1), COLOR_BLOOD_RED);
        }
    }
}

fn draw_soul_essence(img: &mut RgbaImage, col: u32, row: u32) {
    // Floating glowing soul wisp
    fill_tile(img, col, row, COLOR_TRANSPARENT);
    for y in 4..11 {
        for x in 5..11 {
            set_px(img, col, row, x, y, COLOR_MAGIC_BLUE);
        }
    }
    // Core radiance
    for y in 6..9 {
        for x in 6..10 {
            set_px(img, col, row, x, y, COLOR_PURE_WHITE);
        }
    }
    set_px(img, col, row, 7, 3, COLOR_MAGIC_BLUE);
    set_px(img, col, row, 8, 3, COLOR_MAGIC_BLUE);
}

fn draw_explosion_fx(img: &mut RgbaImage, col: u32, row: u32) {
    fill_tile(img, col, row, COLOR_TRANSPARENT);
    // Burst particles
    for i in 2..14 {
        set_px(img, col, row, i, i, COLOR_CRIMSON_RED);
        set_px(img, col, row, i, 15 - i, COLOR_CRIMSON_RED);
    }
    for y in 6..10 {
        for x in 6..10 {
            set_px(img, col, row, x, y, COLOR_SAND_ORANGE);
        }
    }
    set_px(img, col, row, 7, 7, COLOR_PURE_WHITE);
    set_px(img, col, row, 8, 8, COLOR_PURE_WHITE);
}

// ─────────────────────────────────────────────────────────────────────────────
// Corpse, Fog & Overlay Drawers (Row 3)
// ─────────────────────────────────────────────────────────────────────────────

fn draw_corpse_intact(img: &mut RgbaImage, col: u32, row: u32) {
    // Fallen hero lying flat with red blood pool
    fill_tile(img, col, row, COLOR_STONE_GRAY);
    // Blood puddle
    for y in 8..13 {
        for x in 3..14 {
            set_px(img, col, row, x, y, COLOR_BLOOD_RED);
        }
    }
    // Horizontal body silhouette
    for x in 4..12 {
        set_px(img, col, row, x, 9, COLOR_METAL_GRAY);
        set_px(img, col, row, x, 10, COLOR_METAL_GRAY);
    }
    // Helmet on left
    set_px(img, col, row, 3, 9, COLOR_METAL_GRAY);
    set_px(img, col, row, 3, 10, COLOR_METAL_GRAY);
    // Boots on right
    set_px(img, col, row, 13, 9, COLOR_LEATHER_BROWN);
    set_px(img, col, row, 13, 10, COLOR_LEATHER_BROWN);
}

fn draw_corpse_damaged(img: &mut RgbaImage, col: u32, row: u32) {
    fill_tile(img, col, row, COLOR_STONE_GRAY);
    // Large irregular blood splatter
    for y in 6..14 {
        for x in 2..15 {
            if (x + y) % 2 == 0 {
                set_px(img, col, row, x, y, COLOR_BLOOD_RED);
            }
        }
    }
    // Mutilated torso & severed limb
    for x in 5..10 {
        set_px(img, col, row, x, 9, COLOR_METAL_GRAY);
    }
    set_px(img, col, row, 3, 7, COLOR_BONE_WHITE);
    set_px(img, col, row, 12, 11, COLOR_BONE_WHITE);
}

fn draw_corpse_bones(img: &mut RgbaImage, col: u32, row: u32) {
    fill_tile(img, col, row, COLOR_STONE_GRAY);
    // Skull resting on ground
    for y in 7..10 {
        for x in 5..8 {
            set_px(img, col, row, x, y, COLOR_BONE_WHITE);
        }
    }
    set_px(img, col, row, 6, 8, COLOR_VOID);
    // Broken rib fragments
    set_px(img, col, row, 9, 8, COLOR_BONE_WHITE);
    set_px(img, col, row, 11, 7, COLOR_BONE_WHITE);
    set_px(img, col, row, 10, 10, COLOR_BONE_WHITE);
    set_px(img, col, row, 12, 11, COLOR_BONE_WHITE);
    // Bone dust specks
    set_px(img, col, row, 4, 11, COLOR_BONE_WHITE);
    set_px(img, col, row, 7, 12, COLOR_BONE_WHITE);
}

fn draw_fog_unexplored(img: &mut RgbaImage, col: u32, row: u32) {
    // 100% opaque void
    fill_tile(img, col, row, COLOR_VOID);
}

fn draw_fog_explored(img: &mut RgbaImage, col: u32, row: u32) {
    // Dithered semi-transparent mask (checkerboard alpha)
    fill_tile(img, col, row, COLOR_TRANSPARENT);
    let mask = Rgba([0x14, 0x10, 0x13, 0xA0]);
    for y in 0..16 {
        for x in 0..16 {
            if (x + y) % 2 == 0 {
                set_px(img, col, row, x, y, mask);
            }
        }
    }
}

fn draw_intel_compromised(img: &mut RgbaImage, col: u32, row: u32) {
    // Crimson hazard warning overlay (eye glyph)
    fill_tile(img, col, row, COLOR_TRANSPARENT);
    let glow = Rgba([0xA5, 0x30, 0x30, 0x99]);
    let bright = Rgba([0xFF, 0xFF, 0xFF, 0xCC]);
    for x in 4..12 {
        set_px(img, col, row, x, 7, glow);
        set_px(img, col, row, x, 8, glow);
    }
    for y in 5..11 {
        set_px(img, col, row, 7, y, glow);
        set_px(img, col, row, 8, y, glow);
    }
    set_px(img, col, row, 7, 7, bright);
    set_px(img, col, row, 8, 8, bright);
}

fn draw_terror_miasma(img: &mut RgbaImage, col: u32, row: u32) {
    // Purple mystical fog
    fill_tile(img, col, row, COLOR_TRANSPARENT);
    let mist = Rgba([0x3B, 0x17, 0x25, 0x88]);
    for y in 2..14 {
        for x in 2..14 {
            if (x * 3 + y * 7) % 5 == 0 {
                set_px(img, col, row, x, y, mist);
            }
        }
    }
}

fn draw_tile_cursor(img: &mut RgbaImage, col: u32, row: u32) {
    fill_tile(img, col, row, COLOR_TRANSPARENT);
    let border = COLOR_ANCIENT_GOLD;
    // 4 Corner brackets
    for len in 0..4 {
        // Top-left
        set_px(img, col, row, len, 0, border);
        set_px(img, col, row, 0, len, border);
        // Top-right
        set_px(img, col, row, 15 - len, 0, border);
        set_px(img, col, row, 15, len, border);
        // Bottom-left
        set_px(img, col, row, len, 15, border);
        set_px(img, col, row, 0, 15 - len, border);
        // Bottom-right
        set_px(img, col, row, 15 - len, 15, border);
        set_px(img, col, row, 15, 15 - len, border);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public Generator API
// ─────────────────────────────────────────────────────────────────────────────

/// Generates the complete 128x64 `dungeon_sheet` sprite atlas in memory.
#[must_use]
pub fn generate_dungeon_sheet_image() -> RgbaImage {
    let mut img = RgbaImage::new(SHEET_WIDTH, SHEET_HEIGHT);

    // Row 0: Tiles & Decors
    draw_floor_tile(&mut img, 0, 0);
    draw_wall_tile(&mut img, 1, 0);
    draw_stairs_down(&mut img, 2, 0);
    draw_stairs_up(&mut img, 3, 0);
    draw_acid_pit(&mut img, 4, 0);
    draw_portcullis(&mut img, 5, 0, false);
    draw_portcullis(&mut img, 6, 0, true);
    draw_sanctified_floor(&mut img, 7, 0);

    // Row 1: Heroes
    draw_warrior(&mut img, 0, 1);
    draw_cleric(&mut img, 1, 1);
    draw_paladin(&mut img, 2, 1);
    draw_mage(&mut img, 3, 1);
    draw_rogue(&mut img, 4, 1);
    draw_necromancer(&mut img, 5, 1);
    draw_hero_panicked(&mut img, 6, 1);
    draw_hero_retreating(&mut img, 7, 1);

    // Row 2: Monsters & Traps
    draw_skeleton(&mut img, 0, 2);
    draw_zombie(&mut img, 1, 2);
    draw_spectre(&mut img, 2, 2);
    draw_boss(&mut img, 3, 2);
    draw_trap_spikes(&mut img, 4, 2, false);
    draw_trap_spikes(&mut img, 5, 2, true);
    draw_soul_essence(&mut img, 6, 2);
    draw_explosion_fx(&mut img, 7, 2);

    // Row 3: Corpses & Fog & Overlays
    draw_corpse_intact(&mut img, 0, 3);
    draw_corpse_damaged(&mut img, 1, 3);
    draw_corpse_bones(&mut img, 2, 3);
    draw_fog_unexplored(&mut img, 3, 3);
    draw_fog_explored(&mut img, 4, 3);
    draw_intel_compromised(&mut img, 5, 3);
    draw_terror_miasma(&mut img, 6, 3);
    draw_tile_cursor(&mut img, 7, 3);

    img
}

/// Generates the PNG-encoded byte stream of the sprite atlas.
///
/// # Errors
/// Returns an `image::ImageError` if encoding fails.
pub fn generate_dungeon_sheet_png_bytes() -> Result<Vec<u8>, image::ImageError> {
    let img = generate_dungeon_sheet_image();
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    img.write_to(&mut cursor, ImageFormat::Png)?;
    Ok(bytes)
}

/// Ensures the procedural sprite sheet PNG file exists on disk at the given path.
///
/// Automatically creates missing directories and writes the PNG data.
///
/// # Errors
/// Returns an `io::Error` if filesystem operations fail.
pub fn ensure_dungeon_sheet_exists(path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if !path.exists() {
        match generate_dungeon_sheet_png_bytes() {
            Ok(bytes) => {
                let mut file = fs::File::create(path)?;
                file.write_all(&bytes)?;
            }
            Err(e) => {
                return Err(std::io::Error::other(format!("PNG encoding error: {e}")));
            }
        }
    }
    Ok(())
}

/// Embedded raw PNG bytes of the dark & macabre game logo icon (256x256).
pub const GAME_LOGO_ICON_BYTES: &[u8] = include_bytes!("../../../assets/branding/icon_256.png");

/// Returns the embedded 256x256 dark & macabre game logo as a Bevy `Image`.
#[must_use]
pub fn load_game_logo_image() -> bevy::image::Image {
    let (width, height, data) = match image::load_from_memory(GAME_LOGO_ICON_BYTES) {
        Ok(dynamic) => {
            let rgba = dynamic.to_rgba8();
            let (w, h) = rgba.dimensions();
            (w, h, rgba.into_raw())
        }
        Err(_) => (1, 1, vec![0, 0, 0, 255]),
    };
    bevy::image::Image::new(
        bevy::render::render_resource::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
}

/// Extracts a 16x16 tile at (col, row) and scales it 2x (32x32) using nearest-neighbor for sharp UI rendering.
#[must_use]
pub fn extract_tile_image(col: u32, row: u32) -> bevy::image::Image {
    let sheet = generate_dungeon_sheet_image();
    let x_offset = col * TILE_SIZE;
    let y_offset = row * TILE_SIZE;
    let target_size = 32u32;
    let mut rgba_raw = Vec::with_capacity((target_size * target_size * 4) as usize);

    for ty in 0..target_size {
        let sy = y_offset + (ty / 2);
        for tx in 0..target_size {
            let sx = x_offset + (tx / 2);
            let px = sheet.get_pixel(sx, sy);
            rgba_raw.extend_from_slice(&px.0);
        }
    }

    bevy::image::Image::new(
        bevy::render::render_resource::Extent3d {
            width: target_size,
            height: target_size,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        rgba_raw,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_generator_dimensions() {
        let img = generate_dungeon_sheet_image();
        assert_eq!(img.width(), SHEET_WIDTH);
        assert_eq!(img.height(), SHEET_HEIGHT);
        assert_eq!(img.width(), 128);
        assert_eq!(img.height(), 64);
    }

    #[test]
    fn test_asset_generator_png_magic_header() {
        let res = generate_dungeon_sheet_png_bytes();
        assert!(res.is_ok());
        let png_bytes = res.unwrap_or_default();
        assert!(png_bytes.len() > 8);
        assert_eq!(&png_bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]); // \x89PNG
    }

    #[test]
    fn test_game_logo_embedded_bytes() {
        assert!(GAME_LOGO_ICON_BYTES.len() > 1000);
        assert_eq!(&GAME_LOGO_ICON_BYTES[0..4], &[0x89, 0x50, 0x4E, 0x47]);
        let logo_img = load_game_logo_image();
        assert_eq!(logo_img.width(), 256);
        assert_eq!(logo_img.height(), 256);
    }

    #[test]
    fn test_extract_tile_image_dimensions() {
        let tile_img = extract_tile_image(1, 0);
        assert_eq!(tile_img.width(), 32);
        assert_eq!(tile_img.height(), 32);
    }
}
