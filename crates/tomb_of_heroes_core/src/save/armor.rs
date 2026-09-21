//! Base64 ASCII armored export and import pipeline.
//!
//! Conforms to `SPEC-REQ-SAVE-002`.

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;

use crate::save::envelope::SaveEnvelope;
use crate::save::error::SaveError;

/// Formal armor header delimiter.
pub const ARMOR_HEADER: &str = "-----BEGIN TOMB OF HEROES SAVE-----";

/// Formal armor footer delimiter.
pub const ARMOR_FOOTER: &str = "-----END TOMB OF HEROES SAVE-----";

/// Exports a `SaveEnvelope` into an ASCII Base64 armored string.
#[must_use]
pub fn export_to_base64_armor(envelope: &SaveEnvelope) -> String {
    let raw = envelope.to_bytes();
    let encoded = STANDARD.encode(raw);
    format!("{ARMOR_HEADER}\n{encoded}\n{ARMOR_FOOTER}")
}

/// Imports a `SaveEnvelope` from an ASCII Base64 armored string.
///
/// Strips internal whitespace, line feeds, and carriage returns before decoding.
///
/// # Errors
/// Returns `SaveError::Base64Error` if delimiters are missing or Base64 decoding fails.
/// Returns other `SaveError` variants if binary header or payload validation fails.
pub fn import_from_base64_armor(armored: &str) -> Result<SaveEnvelope, SaveError> {
    let start_idx = armored
        .find(ARMOR_HEADER)
        .ok_or_else(|| SaveError::Base64Error("missing armor header delimiter".to_string()))?;
    let content_start = start_idx + ARMOR_HEADER.len();
    let end_relative = armored[content_start..]
        .find(ARMOR_FOOTER)
        .ok_or_else(|| SaveError::Base64Error("missing armor footer delimiter".to_string()))?;
    let base64_slice = &armored[content_start..content_start + end_relative];

    let cleaned: String = base64_slice
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let bytes = STANDARD
        .decode(cleaned.as_bytes())
        .map_err(|e| SaveError::Base64Error(e.to_string()))?;

    SaveEnvelope::from_bytes(&bytes)
}

/// Convenience alias for `export_to_base64_armor`.
#[must_use]
pub fn export_to_base64_string(envelope: &SaveEnvelope) -> String {
    export_to_base64_armor(envelope)
}

/// Convenience alias for `import_from_base64_armor`.
pub fn import_from_base64_string(armored_text: &str) -> Result<SaveEnvelope, SaveError> {
    import_from_base64_armor(armored_text)
}
