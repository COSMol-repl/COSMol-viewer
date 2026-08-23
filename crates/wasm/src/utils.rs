use base64::Engine;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::io::Read;
use std::io::Write;

/// Version marker for the standalone animation payload format.
pub const ANIMATION_PAYLOAD_PREFIX: &str = "CMV1:";

/// Compress data into a base64-encoded string.
pub fn compress_data<T: serde::Serialize>(value: &T) -> Result<String, postcard::Error> {
    let bytes = postcard::to_allocvec(value)?;
    let mut encoder = GzEncoder::new(Vec::with_capacity(bytes.len() / 2), Compression::fast());
    encoder.write_all(&bytes).unwrap();
    let compressed = encoder.finish().unwrap();
    Ok(base64::engine::general_purpose::STANDARD.encode(compressed))
}

/// Compress an animation payload with an explicit format version.
pub fn compress_animation<T: serde::Serialize>(value: &T) -> Result<String, postcard::Error> {
    Ok(format!(
        "{ANIMATION_PAYLOAD_PREFIX}{}",
        compress_data(value)?
    ))
}

#[derive(thiserror::Error, Debug)]
pub enum DecompressError {
    #[error("base64 decode failed: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("gzip decode failed: {0}")]
    Gzip(#[from] std::io::Error),

    #[error("postcard decode failed: {0}")]
    Postcard(#[from] postcard::Error),

    #[error("unsupported animation payload version: {0}")]
    UnsupportedAnimationVersion(String),
}

/// Decompress data from a base64-encoded string.
pub fn decompress_data<T: for<'de> serde::Deserialize<'de>>(s: &str) -> Result<T, DecompressError> {
    let compressed = base64::engine::general_purpose::STANDARD.decode(s)?;
    let mut decoder = GzDecoder::new(&*compressed);
    let mut bytes = Vec::with_capacity(compressed.len());
    decoder.read_to_end(&mut bytes)?;
    Ok(postcard::from_bytes(&bytes)?)
}

/// Decompress a versioned animation payload.
///
/// Unprefixed payloads are accepted for compatibility with payloads produced
/// before the standalone animation format was versioned.
pub fn decompress_animation<T: for<'de> serde::Deserialize<'de>>(
    s: &str,
) -> Result<T, DecompressError> {
    let payload = s.trim();
    if let Some(encoded) = payload.strip_prefix(ANIMATION_PAYLOAD_PREFIX) {
        return decompress_data(encoded);
    }

    if payload.starts_with("CMV") {
        let version = payload.split(':').next().unwrap_or(payload);
        return Err(DecompressError::UnsupportedAnimationVersion(
            version.to_owned(),
        ));
    }

    decompress_data(payload)
}

#[cfg(test)]
mod tests {
    use super::{ANIMATION_PAYLOAD_PREFIX, compress_animation, decompress_animation};

    #[test]
    fn animation_payload_has_a_version_prefix() {
        let payload = compress_animation(&vec![1_u8, 2, 3]).unwrap();
        assert!(payload.starts_with(ANIMATION_PAYLOAD_PREFIX));
        assert_eq!(
            decompress_animation::<Vec<u8>>(&payload).unwrap(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn animation_decoder_accepts_legacy_unprefixed_payloads() {
        let payload = compress_animation(&vec![1_u8, 2, 3]).unwrap();
        let legacy = payload
            .strip_prefix(ANIMATION_PAYLOAD_PREFIX)
            .expect("version prefix");
        assert_eq!(
            decompress_animation::<Vec<u8>>(legacy).unwrap(),
            vec![1, 2, 3]
        );
    }
}
