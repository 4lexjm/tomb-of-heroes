//! # Tomb of Heroes Core (`tomb_of_heroes_core`)
//!
//! Pure, deterministic domain library for the *Tomb of Heroes* engine.
//! This crate is strictly isolated from graphics, audio, or non-synchronous I/O.
//!
//! ## Architectural Invariants
//! - **Strict determinism:** No floating-point numbers (`f32`, `f64`).
//! - **Integer arithmetic:** Percentages and ratios represented in basis points (10_000 = 100.00%).
//! - **Stable identifiers:** Exclusive usage of `LogicId(u64)` for logic and persistence.
//! - **Zero magic numbers:** All engine and gameplay parameters are sourced from [`GameConfig`].
//! - **Exhaustive error handling:** Prohibition of `unwrap()`, `expect()`, and `panic!()`.

#![deny(clippy::float_arithmetic)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

extern crate alloc;

pub mod chrono;
pub mod config;
pub mod hash;
pub mod id;
pub mod math;
pub mod necro;
pub mod rng;
pub mod time;
pub mod topology;
pub mod world;

pub use chrono::*;
pub use config::*;
pub use hash::*;
pub use id::*;
pub use math::*;
pub use necro::*;
pub use rng::*;
pub use time::*;
pub use topology::*;
pub use world::*;
