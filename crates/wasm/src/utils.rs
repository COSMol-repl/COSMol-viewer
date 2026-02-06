use base64::Engine;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::io::Read;
use std::io::Write;
/// Compress data into a base64-encoded string.
pub fn compress_data<T: serde::Serialize>(value: &T) -> Result<String, postcard::Error> {
    let bytes = postcard::to_allocvec(value)?;
    let mut encoder = GzEncoder::new(Vec::with_capacity(bytes.len() / 2), Compression::fast());
    encoder.write_all(&bytes).unwrap();
    let compressed = encoder.finish().unwrap();
    Ok(base64::engine::general_purpose::STANDARD.encode(compressed))
}

#[derive(thiserror::Error, Debug)]
pub enum DecompressError {
    #[error("base64 decode failed: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("gzip decode failed: {0}")]
    Gzip(#[from] std::io::Error),

    #[error("postcard decode failed: {0}")]
    Postcard(#[from] postcard::Error),
}

/// Decompress data from a base64-encoded string.
pub fn decompress_data<T: for<'de> serde::Deserialize<'de>>(s: &str) -> Result<T, DecompressError> {
    let compressed = base64::engine::general_purpose::STANDARD.decode(s)?;
    let mut decoder = GzDecoder::new(&*compressed);
    let mut bytes = Vec::with_capacity(compressed.len());
    decoder.read_to_end(&mut bytes)?;
    Ok(postcard::from_bytes(&bytes)?)
}
