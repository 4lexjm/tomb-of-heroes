//! Discrete 2.5D dungeon topology, oriented vertical links, and A* navigation.
//!
//! Enforces `SPEC-DOMAIN-TOPO` (`docs/specs/02_topologie_brouillard.md`).

pub mod astar;
pub mod coordinates;
pub mod error;
pub mod grid;
pub mod links;

pub use astar::*;
pub use coordinates::*;
pub use error::*;
pub use grid::*;
pub use links::*;
