# Quickstart — Tomb of Heroes

Get the game running in under 5 minutes.

---

## Prerequisites

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Minimum supported version: **Rust 1.80** (stable). The project was built and tested on Rust **1.98**.

Verify:
```bash
rustc --version   # rustc 1.98.x or later
cargo --version
```

### 2. System dependencies (Linux)

Bevy requires a few native libraries for windowing and audio:

```bash
# Ubuntu / Debian
sudo apt-get install -y \
  libasound2-dev libudev-dev libx11-dev libxkbcommon-dev \
  libwayland-dev libxrandr-dev pkg-config

# Fedora / RHEL
sudo dnf install -y \
  alsa-lib-devel libudev-devel libX11-devel libxkbcommon-devel \
  wayland-devel libXrandr-devel pkg-config

# Arch
sudo pacman -S --needed \
  alsa-lib systemd-libs libx11 libxkbcommon wayland libxrandr pkg-config
```

> **macOS / Windows:** No additional native dependencies required. Rust's toolchain is self-contained on those platforms.

---

## Clone & Build

```bash
git clone https://github.com/4lexjm/tomb-of-heroes
cd tomb-of-heroes

# Development build (fast compile, debug symbols)
cargo build -p tomb_of_heroes_app

# Release build (optimized for size — LTO, strip, panic=abort)
cargo build -p tomb_of_heroes_app --release
```

---

## Run

```bash
# From source (debug)
cargo run -p tomb_of_heroes_app

# From source (release)
cargo run -p tomb_of_heroes_app --release

# Or run the compiled binary directly
./target/release/tomb_of_heroes_app        # Linux / macOS
.\target\release\tomb_of_heroes_app.exe   # Windows
```

The window opens at your native resolution. The engine automatically computes the highest integer scaling factor that fits the **320×240 Safe Zone** without distortion.

---

## Verify the Build

Run the full test suite to confirm everything is working:

```bash
cargo test --workspace
# Expected: 113 tests, 0 failures, 0 warnings
```

---

## Useful Commands

```bash
# Lint (zero warnings policy)
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --check

# Auto-format
cargo fmt

# Check only (no artifact output, fastest feedback)
cargo check-core   # checks tomb_of_heroes_core
cargo check-app    # checks tomb_of_heroes_app
```

> These aliases are defined in [`.cargo/config.toml`](.cargo/config.toml).

---

## Headless (No Window)

The simulation core runs entirely without a window. You can drive it programmatically:

```rust
use tomb_of_heroes_core::{GameConfig, LogicWorld};
use tomb_of_heroes_core::rng::DungeonMasterSeed;

let config = GameConfig::default();
let mut world = LogicWorld::new(config, DungeonMasterSeed(12345));

for _ in 0..20 {
    world.step(); // advance one fixed tick (50 ms)
}

println!("Tick: {:?}", world.current_tick()); // Tick(20)
println!("Hash: {:?}", world.state_hash());
```

Or run tests without any display:

```bash
cargo test -p tomb_of_heroes_core   # pure headless, no window needed
cargo test -p tomb_of_heroes_app    # Bevy MinimalPlugins, also headless
```

---

## Save & Export

Saves are binary envelopes compressed with Zstd. Export a save as portable Base64 text:

```rust
use tomb_of_heroes_core::save::{pack_world, export_to_base64_armor};

let envelope = pack_world(&world, campaign_id, 3)?;
let armored  = export_to_base64_armor(&envelope);
// → "-----BEGIN TOMB OF HEROES SAVE-----\n...\n-----END TOMB OF HEROES SAVE-----"
```

Paste the armor block anywhere (clipboard, file, pastebin) to transfer your run between devices.

---

## Troubleshooting

| Issue | Fix |
|---|---|
| `cargo: command not found` | Run `source "$HOME/.cargo/env"` or restart your shell |
| Linker errors on Linux | Install the system dependencies listed above |
| Black screen / no window | Ensure your GPU driver supports Vulkan or OpenGL 3.3+ |
| `DISPLAY` / Wayland errors in CI | Use `WINIT_UNIX_BACKEND=x11` or run headless tests (`cargo test -p tomb_of_heroes_core`) |
| Slow first build | Bevy compiles many dependencies; subsequent builds are cached and fast |
