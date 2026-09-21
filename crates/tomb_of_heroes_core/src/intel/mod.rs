//! Intelligence, squad retreat decisions, guild register, and veterancy module.
//!
//! Conforms to `SPEC-DOMAIN-INTEL` (`docs/specs/04_fuite_renseignement_veterance.md`).

pub(crate) mod error;
pub mod guild;
pub mod retreat;
pub mod veterancy;

pub use error::*;
pub use guild::*;
pub use retreat::*;
pub use veterancy::*;
