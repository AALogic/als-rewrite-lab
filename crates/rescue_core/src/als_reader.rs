use std::path::Path;

use crate::{ALSError, ALSReadModel};

pub const ALS_READ_MODEL_VERSION: &str = "0.2";
pub const ALS_READER_VERSION: &str = "0.2.2";
pub const MAX_COMPRESSED_ALS_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_DECOMPRESSED_XML_BYTES: u64 = 1024 * 1024 * 1024;

pub fn analyze_als(path: impl AsRef<Path>) -> Result<ALSReadModel, ALSError> {
    crate::als_reader_impl::analyze_als_impl(path.as_ref())
}
