//! Necromancy, corpses, and terror systemic module.
//!
//! Conforms to `SPEC-DOMAIN-NECRO` (`docs/specs/03_cadavres_terror_necromancie.md`).

pub mod corpse;
pub(crate) mod error;
pub mod necromancy;
pub mod spatial;
pub mod terror;

pub use corpse::*;
pub use error::*;
pub use necromancy::*;
pub use spatial::*;
pub use terror::*;
