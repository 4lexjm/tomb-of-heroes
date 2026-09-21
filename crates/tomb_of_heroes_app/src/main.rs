//! # Tomb of Heroes Application Entry Point
//!
//! Launches the Bevy 0.15 graphical host for *Tomb of Heroes*.
//!
//! The `#[bevy_main]` attribute macro handles platform-specific entry points:
//! - **Desktop (Linux/macOS/Windows):** expands to a standard `fn main()`.
//! - **Android:** expands to `android_main(android_app: AndroidApp)`, wiring
//!   `winit`'s android activity before calling into Bevy's event loop.
//! - **iOS:** expands to `fn main()` with UIKit integration set up by winit.

use bevy::prelude::bevy_main;
use tomb_of_heroes_app::build_app;

#[bevy_main]
fn main() {
    build_app().run();
}
