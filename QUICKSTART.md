# Quickstart — Tomb of Heroes

Get the game running on your device. The primary targets are **Android** and **iOS** — desktop builds are available for development and testing.

---

## ⚡ Automated Builds (GitHub Actions)

If you do not want to set up local mobile SDKs and cross-compilers, **pre-built release artifacts are generated on every push** to `main`:

1. Go to the **Actions** tab on GitHub.
2. Select the latest **CI & Mobile Artifacts Build** run.
3. Scroll down to **Artifacts** to download:
   - 🤖 `tomb-of-heroes-android-apk` : Release APK (ARM64) ready to install on Android (`adb install tomb_of_heroes_app.apk`).
   - 🍏 `tomb-of-heroes-ios-simulator-app` : `.app.zip` bundle for Apple Silicon iOS Simulator (`xcrun simctl install booted TombOfHeroes.app`).
   - 📱 `tomb-of-heroes-ios-device-unsigned-ipa` : Unsigned IPA for physical device sideloading via AltStore, Sideloadly, or TrollStore.

---

## Prerequisites

### Rust toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Minimum supported version: **Rust 1.80** (stable). Built and tested on Rust **1.98**.

```bash
rustc --version   # rustc 1.98.x or later
```

---

## 🤖 Android

### 1. Install dependencies

```bash
# Android SDK + NDK (via Android Studio recommended)
# Then install the Rust Android targets:
rustup target add \
  aarch64-linux-android \
  armv7-linux-androideabi \
  x86_64-linux-android

# Install cargo-apk
cargo install cargo-apk
```

Set your SDK/NDK paths (adjust versions to match your install):

```bash
export ANDROID_SDK_ROOT="$HOME/Android/Sdk"
export ANDROID_NDK_ROOT="$ANDROID_SDK_ROOT/ndk/26.1.10909125"
```

### 2. Build the APK

```bash
# Debug APK
cargo apk build -p tomb_of_heroes_app --lib

# Release APK (optimized: opt-level=z, LTO, stripped)
cargo apk build -p tomb_of_heroes_app --lib --release
```

The APK is output to `target/release/apk/tomb_of_heroes_app.apk`.

### 3. Install & run on a connected device

```bash
# Connect an Android device with USB debugging enabled, then:
cargo apk run -p tomb_of_heroes_app --release
```

Or install the APK manually:
```bash
adb install target/release/apk/tomb_of_heroes_app.apk
```

---

## 🍎 iOS / iPadOS

### 1. Requirements

- macOS with **Xcode 15+** installed
- Apple Developer account (free tier works for device sideloading)

### 2. Install dependencies

```bash
# iOS targets
rustup target add \
  aarch64-apple-ios \
  aarch64-apple-ios-sim \
  x86_64-apple-ios

# cargo-bundle for Xcode project generation
cargo install cargo-bundle
```

### 3. Build for iOS Simulator

```bash
cargo build -p tomb_of_heroes_app \
  --target aarch64-apple-ios-sim \
  --release
```

### 4. Build for physical device

```bash
cargo build -p tomb_of_heroes_app \
  --target aarch64-apple-ios \
  --release
```

Then generate the `.app` bundle and open in Xcode to sign and deploy:

```bash
cargo bundle --target aarch64-apple-ios --release
open target/aarch64-apple-ios/release/bundle/ios/tomb_of_heroes_app.app
```

> Xcode handles code signing. Select your development team under *Signing & Capabilities* and click *Run* to install on a connected iPhone or iPad.

---

## 🖥️ Desktop (Development & CI)

Desktop builds are used for local development, running tests, and CI. They are **not the shipping target**.

### System dependencies (Linux only)

```bash
# Ubuntu / Debian
sudo apt-get install -y \
  libasound2-dev libudev-dev libx11-dev libxkbcommon-dev \
  libwayland-dev libxrandr-dev pkg-config

# Fedora
sudo dnf install -y \
  alsa-lib-devel libudev-devel libX11-devel libxkbcommon-devel \
  wayland-devel libXrandr-devel pkg-config

# Arch
sudo pacman -S --needed \
  alsa-lib systemd-libs libx11 libxkbcommon wayland libxrandr pkg-config
```

> **macOS / Windows:** No additional dependencies.

### Build & run

```bash
cargo run -p tomb_of_heroes_app           # debug
cargo run -p tomb_of_heroes_app --release # release
```

---

## ✅ Verify the build

Run the full test suite (headless, no window, no device required):

```bash
cargo test --workspace
# Expected: 113 tests, 0 failures, 0 warnings
```

All tests in `tomb_of_heroes_core` are pure headless. App tests use `MinimalPlugins` — no GPU or screen needed.

---

## Useful commands

```bash
# Lint (zero warnings policy)
cargo clippy --workspace --all-targets -- -D warnings

# Format
cargo fmt --check
cargo fmt          # auto-fix

# Aliases (defined in .cargo/config.toml)
cargo check-core   # cargo check -p tomb_of_heroes_core
cargo check-app    # cargo check -p tomb_of_heroes_app
cargo lint-all     # clippy -D warnings
```

---

## Save transfer between devices

Saves are exportable as a plain-text Base64 block — paste it into a message, email, or cloud note to transfer between your phone and tablet:

```
-----BEGIN TOMB OF HEROES SAVE-----
VE9IUwEA...
-----END TOMB OF HEROES SAVE-----
```

The game verifies CRC32 integrity and `StateHash` on import, so a corrupted or tampered block is rejected before any state is mutated.

---

## Troubleshooting

| Issue | Fix |
|---|---|
| `cargo: command not found` | Run `source "$HOME/.cargo/env"` or restart your shell |
| `cargo-apk` build fails | Verify `ANDROID_SDK_ROOT` and `ANDROID_NDK_ROOT` are set and point to matching NDK r23+ |
| iOS: code signing error | Open in Xcode, set your Team under *Signing & Capabilities* |
| iOS Simulator: `arch` mismatch | Use `aarch64-apple-ios-sim` for Apple Silicon Macs, `x86_64-apple-ios` for Intel |
| Linux desktop: linker errors | Install the system dependencies listed above |
| Black screen on device | Ensure Vulkan/GLES3 is supported; check `adb logcat` for Bevy/wgpu errors |
| Slow first build | Bevy compiles many dependencies on first build; subsequent builds use the cache and are fast |
