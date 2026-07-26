use crate::ALSError;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) struct DecodedAls {
    pub(crate) source_path: String,
    pub(crate) source_file_size: u64,
    pub(crate) source_file_hash: String,
    pub(crate) xml: String,
}

pub(crate) fn decode_als(path: &Path) -> Result<DecodedAls, ALSError> {
    let source_path = path.to_string_lossy().to_string();
    if !path.exists() {
        return Err(ALSError::FileNotFound { path: source_path });
    }

    let compressed = read_compressed_als(path, &source_path)?;
    let source_file_size = compressed.len() as u64;
    let source_file_hash = sha256_hex(&compressed);
    let xml_bytes = decompress_xml_bytes(&compressed, &source_path)?;
    let xml = String::from_utf8(xml_bytes).map_err(|error| ALSError::InvalidXml {
        path: source_path.clone(),
        message: error.to_string(),
    })?;

    Ok(DecodedAls {
        source_path,
        source_file_size,
        source_file_hash,
        xml,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn read_compressed_als(path: &Path, source_path: &str) -> Result<Vec<u8>, ALSError> {
    let metadata = fs::metadata(path).map_err(|error| ALSError::FileNotReadable {
        path: source_path.to_string(),
        message: error.to_string(),
    })?;

    if metadata.len() > crate::als_reader::MAX_COMPRESSED_ALS_BYTES {
        return Err(ALSError::CompressedTooLarge {
            path: source_path.to_string(),
            size: metadata.len(),
            limit: crate::als_reader::MAX_COMPRESSED_ALS_BYTES,
        });
    }

    fs::read(path).map_err(|error| ALSError::FileNotReadable {
        path: source_path.to_string(),
        message: error.to_string(),
    })
}

fn decompress_xml_bytes(compressed: &[u8], source_path: &str) -> Result<Vec<u8>, ALSError> {
    decompress_xml_bytes_with_limit(
        compressed,
        source_path,
        crate::als_reader::MAX_DECOMPRESSED_XML_BYTES,
    )
}

fn decompress_xml_bytes_with_limit(
    compressed: &[u8],
    source_path: &str,
    max_decompressed_bytes: u64,
) -> Result<Vec<u8>, ALSError> {
    let mut decoder = flate2::read::GzDecoder::new(compressed);
    let mut limited = decoder
        .by_ref()
        .take(max_decompressed_bytes.saturating_add(1));
    let mut xml_bytes = Vec::new();

    limited
        .read_to_end(&mut xml_bytes)
        .map_err(|_| ALSError::NotGzip {
            path: source_path.to_string(),
        })?;

    if xml_bytes.len() as u64 > max_decompressed_bytes {
        return Err(ALSError::DecompressedXmlTooLarge {
            path: source_path.to_string(),
            limit: max_decompressed_bytes,
        });
    }

    Ok(xml_bytes)
}

pub(crate) fn unix_timestamp_millis() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::decompress_xml_bytes_with_limit;
    use crate::ALSError;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn decompressed_xml_size_limit_is_enforced() -> Result<(), std::io::Error> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"<Ableton><LiveSet /></Ableton>")?;
        let compressed = encoder.finish()?;
        let result = decompress_xml_bytes_with_limit(&compressed, "limit-test.als", 8);

        assert!(matches!(
            result,
            Err(ALSError::DecompressedXmlTooLarge { .. })
        ));
        Ok(())
    }
}
