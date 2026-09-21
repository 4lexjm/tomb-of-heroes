//! Domain error definitions for save envelopes and serialization.
//!
//! Conforms to `SPEC-REQ-SAVE-001` and `SPEC-REQ-SAVE-002`.

use std::fmt;

/// Domain errors encountered during save envelope packing, unpacking, or armor conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveError {
    /// Save header magic bytes do not match the expected formal constant (`TOHS`).
    InvalidMagic,
    /// Save envelope payload is empty (0 bytes).
    EmptyPayload,
    /// Size of payload does not match header declaration.
    SizeMismatch,
    /// CRC32 checksum mismatch on decompressed payload.
    CrcMismatch,
    /// Zstd decompression failure.
    DecompressionFailed(String),
    /// Deserialization into deterministic simulation state failed.
    DeserializationFailed(String),
    /// StateHash post-load integrity verification mismatch.
    IntegrityViolation,
    /// Base64 armor parsing or encoding error.
    Base64Error(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "invalid save magic bytes (expected TOHS)"),
            Self::EmptyPayload => write!(f, "save payload is empty"),
            Self::SizeMismatch => write!(f, "payload size does not match expected size in header"),
            Self::CrcMismatch => write!(f, "CRC32 checksum mismatch on decompressed payload"),
            Self::DecompressionFailed(msg) => write!(f, "zstd decompression failed: {msg}"),
            Self::DeserializationFailed(msg) => write!(f, "state deserialization failed: {msg}"),
            Self::IntegrityViolation => write!(f, "state hash integrity verification failed"),
            Self::Base64Error(msg) => write!(f, "base64 armor error: {msg}"),
        }
    }
}

impl std::error::Error for SaveError {}
