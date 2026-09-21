//! SaveHeader binary structure and deterministic little-endian layout.
//!
//! Conforms to `SPEC-REQ-SAVE-001`.

use crate::save::error::SaveError;

/// Formal magic constant `TOHS` ("Tomb Of Heroes Save").
pub const SAVE_MAGIC: [u8; 4] = *b"TOHS";

/// Format version 1.
pub const SAVE_FORMAT_VERSION_V1: u16 = 1;

/// Fixed binary size in bytes of `SaveHeader` (4 + 2 + 2 + 16 + 8 + 4 + 4 + 4 + 8 = 52 bytes).
pub const SAVE_HEADER_SIZE: usize = 52;

/// Flag: Payload is compressed with Zstandard (Bit 0).
pub const FLAG_ZSTD: u16 = 1 << 0;

/// Flag: Payload is encrypted (Bit 1).
pub const FLAG_ENCRYPTED: u16 = 1 << 1;

/// Flag: Payload is base64 encoded (Bit 2).
pub const FLAG_BASE64: u16 = 1 << 2;

/// Deterministic binary header for Tomb of Heroes save files.
///
/// Encoded strictly in little-endian. Total binary size: 52 bytes.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveHeader {
    /// Magic signature `[b'T', b'O', b'H', b'S']`.
    pub magic: [u8; 4],
    /// Save format version (v1 = 0x0001).
    pub format_version: u16,
    /// Format and processing flags.
    pub flags: u16,
    /// UUID v4 of active campaign.
    pub campaign_id: [u8; 16],
    /// Simulation tick timestamp.
    pub save_tick: u64,
    /// Size in bytes before compression.
    pub uncompressed_size: u32,
    /// Size in bytes of compressed payload.
    pub compressed_size: u32,
    /// CRC32 checksum (IEEE 802.3) of uncompressed payload.
    pub payload_crc32: u32,
    /// Deterministic 64-bit StateHash for integrity check.
    pub state_hash: u64,
}

impl SaveHeader {
    /// Fixed binary size in bytes.
    pub const SIZE: usize = SAVE_HEADER_SIZE;

    /// Encodes header into a fixed-size byte array in little-endian.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4..6].copy_from_slice(&self.format_version.to_le_bytes());
        bytes[6..8].copy_from_slice(&self.flags.to_le_bytes());
        bytes[8..24].copy_from_slice(&self.campaign_id);
        bytes[24..32].copy_from_slice(&self.save_tick.to_le_bytes());
        bytes[32..36].copy_from_slice(&self.uncompressed_size.to_le_bytes());
        bytes[36..40].copy_from_slice(&self.compressed_size.to_le_bytes());
        bytes[40..44].copy_from_slice(&self.payload_crc32.to_le_bytes());
        bytes[44..52].copy_from_slice(&self.state_hash.to_le_bytes());
        bytes
    }

    /// Decodes header from a byte slice in little-endian.
    ///
    /// # Errors
    /// Returns `SaveError::InvalidMagic` if magic bytes do not match `TOHS`.
    /// Returns `SaveError::SizeMismatch` if `bytes.len()` is less than `SAVE_HEADER_SIZE`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SaveError> {
        if bytes.len() < 4 || bytes[0..4] != SAVE_MAGIC {
            return Err(SaveError::InvalidMagic);
        }
        if bytes.len() < Self::SIZE {
            return Err(SaveError::SizeMismatch);
        }

        let mut magic = [0u8; 4];
        magic.copy_from_slice(&bytes[0..4]);

        let format_version = u16::from_le_bytes([bytes[4], bytes[5]]);
        let flags = u16::from_le_bytes([bytes[6], bytes[7]]);

        let mut campaign_id = [0u8; 16];
        campaign_id.copy_from_slice(&bytes[8..24]);

        let mut tick_bytes = [0u8; 8];
        tick_bytes.copy_from_slice(&bytes[24..32]);
        let save_tick = u64::from_le_bytes(tick_bytes);

        let mut uncompressed_size_bytes = [0u8; 4];
        uncompressed_size_bytes.copy_from_slice(&bytes[32..36]);
        let uncompressed_size = u32::from_le_bytes(uncompressed_size_bytes);

        let mut compressed_size_bytes = [0u8; 4];
        compressed_size_bytes.copy_from_slice(&bytes[36..40]);
        let compressed_size = u32::from_le_bytes(compressed_size_bytes);

        let mut crc_bytes = [0u8; 4];
        crc_bytes.copy_from_slice(&bytes[40..44]);
        let payload_crc32 = u32::from_le_bytes(crc_bytes);

        let mut hash_bytes = [0u8; 8];
        hash_bytes.copy_from_slice(&bytes[44..52]);
        let state_hash = u64::from_le_bytes(hash_bytes);

        Ok(Self {
            magic,
            format_version,
            flags,
            campaign_id,
            save_tick,
            uncompressed_size,
            compressed_size,
            payload_crc32,
            state_hash,
        })
    }
}
