use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::{Compression, GzBuilder};
use rescue_core::{MAX_COMPRESSED_ALS_BYTES, MAX_DECOMPRESSED_XML_BYTES};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub(crate) struct DecodedStagedAls {
    pub compressed_hash: String,
    pub xml: String,
}

pub(crate) struct RewriteIoFailure {
    pub code: &'static str,
    pub message: String,
    pub path: Option<PathBuf>,
}

pub(crate) fn read_staged_als(path: &Path) -> Result<DecodedStagedAls, RewriteIoFailure> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        failure(
            "REWRITE_STAGED_ALS_UNAVAILABLE",
            format!("Cannot inspect staged ALS: {error}"),
            Some(path.to_path_buf()),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(failure(
            "REWRITE_STAGED_ALS_NOT_REGULAR",
            "Staged ALS must be a regular non-symlink file".to_string(),
            Some(path.to_path_buf()),
        ));
    }
    if metadata.len() > MAX_COMPRESSED_ALS_BYTES {
        return Err(failure(
            "REWRITE_STAGED_ALS_TOO_LARGE",
            "Staged ALS exceeds the compressed input limit".to_string(),
            Some(path.to_path_buf()),
        ));
    }
    let compressed = fs::read(path).map_err(|error| {
        failure(
            "REWRITE_STAGED_ALS_READ_FAILED",
            format!("Cannot read staged ALS: {error}"),
            Some(path.to_path_buf()),
        )
    })?;
    let xml_bytes = decompress_limited(&compressed, path)?;
    let xml = String::from_utf8(xml_bytes).map_err(|error| {
        failure(
            "REWRITE_XML_INVALID_UTF8",
            format!("Decompressed ALS XML is not UTF-8: {error}"),
            Some(path.to_path_buf()),
        )
    })?;
    Ok(DecodedStagedAls {
        compressed_hash: sha256_hex(&compressed),
        xml,
    })
}

pub(crate) fn encode_xml(xml: &str) -> Result<Vec<u8>, RewriteIoFailure> {
    let mut encoder: GzEncoder<Vec<u8>> = GzBuilder::new()
        .mtime(0)
        .operating_system(255)
        .write(Vec::new(), Compression::default());
    encoder.write_all(xml.as_bytes()).map_err(|error| {
        failure(
            "REWRITE_GZIP_ENCODE_FAILED",
            format!("Cannot encode rewritten ALS: {error}"),
            None,
        )
    })?;
    encoder.finish().map_err(|error| {
        failure(
            "REWRITE_GZIP_ENCODE_FAILED",
            format!("Cannot finish rewritten ALS encoding: {error}"),
            None,
        )
    })
}

pub(crate) fn validate_encoded_als(compressed: &[u8], path: &Path) -> Result<(), RewriteIoFailure> {
    let xml_bytes = decompress_limited(compressed, path)?;
    let xml = std::str::from_utf8(&xml_bytes).map_err(|error| {
        failure(
            "REWRITE_OUTPUT_INVALID_UTF8",
            format!("Rewritten ALS XML is not UTF-8: {error}"),
            Some(path.to_path_buf()),
        )
    })?;
    let document = roxmltree::Document::parse(xml).map_err(|error| {
        failure(
            "REWRITE_OUTPUT_XML_INVALID",
            format!("Rewritten ALS XML is invalid: {error}"),
            Some(path.to_path_buf()),
        )
    })?;
    if !document.root_element().has_tag_name("Ableton") {
        return Err(failure(
            "REWRITE_OUTPUT_ROOT_INVALID",
            "Rewritten ALS has no Ableton root".to_string(),
            Some(path.to_path_buf()),
        ));
    }
    Ok(())
}

pub(crate) fn replace_staged_als(target: &Path, compressed: &[u8]) -> Result<(), RewriteIoFailure> {
    let temp = temporary_path(target);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| {
            failure(
                "REWRITE_TEMP_CREATE_FAILED",
                format!("Cannot create rewritten ALS temporary file: {error}"),
                Some(temp.clone()),
            )
        })?;
    if let Err(error) = file.write_all(compressed).and_then(|_| file.sync_all()) {
        remove_own_temp(&temp);
        return Err(failure(
            "REWRITE_TEMP_WRITE_FAILED",
            format!("Cannot persist rewritten ALS temporary file: {error}"),
            Some(temp),
        ));
    }
    promote_replacement(&temp, target).map_err(|error| {
        remove_own_temp(&temp);
        failure(
            "REWRITE_ATOMIC_REPLACE_FAILED",
            error,
            Some(target.to_path_buf()),
        )
    })
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn decompress_limited(compressed: &[u8], path: &Path) -> Result<Vec<u8>, RewriteIoFailure> {
    let mut decoder = GzDecoder::new(compressed);
    let mut limited = decoder
        .by_ref()
        .take(MAX_DECOMPRESSED_XML_BYTES.saturating_add(1));
    let mut xml = Vec::new();
    limited.read_to_end(&mut xml).map_err(|error| {
        failure(
            "REWRITE_INPUT_NOT_GZIP",
            format!("Cannot decompress staged ALS: {error}"),
            Some(path.to_path_buf()),
        )
    })?;
    if xml.len() as u64 > MAX_DECOMPRESSED_XML_BYTES {
        return Err(failure(
            "REWRITE_XML_TOO_LARGE",
            "Decompressed ALS exceeds the XML size limit".to_string(),
            Some(path.to_path_buf()),
        ));
    }
    Ok(xml)
}

#[cfg(unix)]
fn promote_replacement(temp: &Path, target: &Path) -> Result<(), String> {
    fs::rename(temp, target).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
fn promote_replacement(_temp: &Path, _target: &Path) -> Result<(), String> {
    Err("Atomic replacement of an existing staged ALS is not implemented on this platform".into())
}

fn temporary_path(target: &Path) -> PathBuf {
    let filename = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("set.als");
    target.with_file_name(format!(".{filename}.rescue-rewrite.tmp"))
}

fn remove_own_temp(path: &Path) {
    let _ = fs::remove_file(path);
}

fn failure(code: &'static str, message: String, path: Option<PathBuf>) -> RewriteIoFailure {
    RewriteIoFailure {
        code,
        message,
        path,
    }
}
