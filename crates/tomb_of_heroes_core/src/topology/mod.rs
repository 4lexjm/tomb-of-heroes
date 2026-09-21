//! Discrete 2.5D dungeon topology, oriented vertical links, and A* navigation.
//!
//! Enforces `SPEC-DOMAIN-TOPO` (`docs/specs/02_topologie_brouillard.md`).

pub mod astar;
pub mod coordinates;
pub(crate) mod error;
pub mod frontier;
pub mod grid;
pub mod knowledge;
pub mod links;
pub mod shadowcasting;

pub use astar::*;
pub use coordinates::*;
pub use error::*;
pub use frontier::*;
pub use grid::*;
pub use knowledge::*;
pub use links::*;
pub use shadowcasting::*;
