//! SaveEnvelope container, binary packing and unpacking pipeline.
//!
//! Conforms to `SPEC-REQ-SAVE-001`.

use crate::save::error::SaveError;
use crate::save::header::{SaveHeader, FLAG_ZSTD, SAVE_FORMAT_VERSION_V1, SAVE_MAGIC};
use crate::world::LogicWorld;

/// Default zstd compression level for fast mobile execution.
pub const DEFAULT_COMPRESSION_LEVEL: i32 = 3;

/// Default empty campaign identifier.
pub const DEFAULT_CAMPAIGN_ID: [u8; 16] = [0u8; 16];

/// Save envelope containing formal header and compressed payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveEnvelope {
    /// Save header with metadata and integrity checksums.
    pub header: SaveHeader,
    /// Compressed payload bytes.
    pub payload: Vec<u8>,
}

impl SaveEnvelope {
    /// Encodes the complete envelope (header followed by payload) to binary.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(SaveHeader::SIZE + self.payload.len());
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    /// Decodes an envelope from binary bytes.
    ///
    /// # Errors
    /// Returns `SaveError::InvalidMagic` if magic bytes do not match `TOHS`.
    /// Returns `SaveError::SizeMismatch` if total bytes is less than the header size or payload size mismatches.
    /// Returns `SaveError::EmptyPayload` if the payload segment has 0 bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SaveError> {
        if bytes.len() < 4 || bytes[0..4] != SAVE_MAGIC {
            return Err(SaveError::InvalidMagic);
        }
        if bytes.len() < SaveHeader::SIZE {
            return Err(SaveError::SizeMismatch);
        }

        let header = SaveHeader::from_bytes(&bytes[0..SaveHeader::SIZE])?;
        let payload = bytes[SaveHeader::SIZE..].to_vec();

        if payload.is_empty() {
            return Err(SaveError::EmptyPayload);
        }
        if payload.len() != header.compressed_size as usize {
            return Err(SaveError::SizeMismatch);
        }

        Ok(Self { header, payload })
    }
}

/// Packs a `LogicWorld` into a `SaveEnvelope` with zstd compression.
///
/// # Errors
/// Returns `SaveError::DeserializationFailed` if JSON serialization fails.
/// Returns `SaveError::DecompressionFailed` if zstd compression fails.
pub fn pack_world(
    world: &LogicWorld,
    campaign_id: [u8; 16],
    compression_level: i32,
) -> Result<SaveEnvelope, SaveError> {
    let uncompressed_bytes =
        serde_json::to_vec(world).map_err(|e| SaveError::DeserializationFailed(e.to_string()))?;

    let uncompressed_size = match u32::try_from(uncompressed_bytes.len()) {
        Ok(sz) => sz,
        Err(_) => return Err(SaveError::SizeMismatch),
    };

    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&uncompressed_bytes);
    let payload_crc32 = hasher.finalize();

    let compressed_payload = zstd::encode_all(&uncompressed_bytes[..], compression_level)
        .map_err(|e| SaveError::DecompressionFailed(e.to_string()))?;

    let compressed_size = match u32::try_from(compressed_payload.len()) {
        Ok(sz) => sz,
        Err(_) => return Err(SaveError::SizeMismatch),
    };

    let state_hash = world.state_hash().as_u64();
    let save_tick = world.current_tick().as_u64();

    let header = SaveHeader {
        magic: SAVE_MAGIC,
        format_version: SAVE_FORMAT_VERSION_V1,
        flags: FLAG_ZSTD,
        campaign_id,
        save_tick,
        uncompressed_size,
        compressed_size,
        payload_crc32,
        state_hash,
    };

    Ok(SaveEnvelope {
        header,
        payload: compressed_payload,
    })
}

/// Unpacks a `SaveEnvelope` into a `LogicWorld`.
///
/// Follows the strict validation pipeline:
/// 1. Verify magic bytes `TOHS`.
/// 2. Verify payload is not empty.
/// 3. Verify payload length matches `header.compressed_size`.
/// 4. Decompress payload using `zstd::decode_all`.
/// 5. Verify decompressed length matches `header.uncompressed_size`.
/// 6. Calculate CRC32 of decompressed bytes and verify equality with `header.payload_crc32`.
/// 7. Deserialize `LogicWorld` from JSON.
/// 8. Recalculate `state_hash` and verify equality with `header.state_hash`.
///
/// # Errors
/// Returns explicit `SaveError` variants on any integrity or format mismatch.
pub fn unpack_world(envelope: &SaveEnvelope) -> Result<LogicWorld, SaveError> {
    // 1. Magic check
    if envelope.header.magic != SAVE_MAGIC {
        return Err(SaveError::InvalidMagic);
    }

    // 2. Empty payload check
    if envelope.payload.is_empty() {
        return Err(SaveError::EmptyPayload);
    }

    // 3. Compressed size match check
    if envelope.payload.len() != envelope.header.compressed_size as usize {
        return Err(SaveError::SizeMismatch);
    }

    // 4. Decompress zstd payload
    let decompressed = zstd::decode_all(&envelope.payload[..])
        .map_err(|e| SaveError::DecompressionFailed(e.to_string()))?;

    // 5. Uncompressed size match check
    if decompressed.len() != envelope.header.uncompressed_size as usize {
        return Err(SaveError::SizeMismatch);
    }

    // 6. CRC32 checksum verification (IEEE 802.3)
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&decompressed);
    let crc = hasher.finalize();
    if crc != envelope.header.payload_crc32 {
        return Err(SaveError::CrcMismatch);
    }

    // 7. Deserialize LogicWorld
    let world: LogicWorld = serde_json::from_slice(&decompressed)
        .map_err(|e| SaveError::DeserializationFailed(e.to_string()))?;

    // 8. Recalculate and verify StateHash integrity
    if world.state_hash().as_u64() != envelope.header.state_hash {
        return Err(SaveError::IntegrityViolation);
    }

    Ok(world)
}

/// Convenience alias for packing a world with default campaign identifier and compression level.
pub fn pack_save_envelope(world: &LogicWorld) -> Result<SaveEnvelope, SaveError> {
    pack_world(world, DEFAULT_CAMPAIGN_ID, DEFAULT_COMPRESSION_LEVEL)
}

/// Convenience alias for unpacking a save envelope.
pub fn unpack_save_envelope(envelope: &SaveEnvelope) -> Result<LogicWorld, SaveError> {
    unpack_world(envelope)
}
