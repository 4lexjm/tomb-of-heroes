//! SaveEnvelope, Zstd persistence, and Base64 armored export.
//!
//! Conforms to `SPEC-DOMAIN-SAVE-VIEW` (`docs/specs/06_sauvegardes_multiratios.md`).

pub mod armor;
pub mod envelope;
pub mod error;
pub mod header;

pub use armor::*;
pub use envelope::*;
pub use error::*;
pub use header::*;
