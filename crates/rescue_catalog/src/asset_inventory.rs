use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const ASSET_INVENTORY_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetInventoryRequest {
    pub scan_run_id: String,
    pub roots: Vec<PathBuf>,
    pub max_entries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetInventoryResult {
    pub metadata: AssetInventoryMetadata,
    pub file_occurrences: Vec<FileOccurrence>,
    pub content_records: Vec<ContentRecord>,
    pub warnings: Vec<AssetInventoryWarning>,
    pub errors: Vec<AssetInventoryError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetInventoryMetadata {
    pub inventory_version: String,
    pub scan_run_id: String,
    pub scan_status: String,
    pub requested_root_count: usize,
    pub scanned_root_count: usize,
    pub entries_visited: usize,
    pub audio_file_count: usize,
    pub content_record_count: usize,
    pub skipped_symlink_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileOccurrence {
    pub file_occurrence_id: String,
    pub content_id: String,
    pub source_root: PathBuf,
    pub native_path: PathBuf,
    pub relative_path: PathBuf,
    pub filename: String,
    pub extension: String,
    pub file_size: u64,
    pub entry_kind: String,
    pub observation_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentRecord {
    pub content_id: String,
    pub hash_algorithm: String,
    pub digest: String,
    pub file_size: u64,
    pub occurrence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetInventoryWarning {
    pub warning_id: usize,
    pub warning_code: String,
    pub message: String,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetInventoryError {
    pub error_code: String,
    pub message: String,
    pub path: Option<PathBuf>,
}

pub fn scan_assets(request: &AssetInventoryRequest) -> AssetInventoryResult {
    crate::asset_inventory_impl::scan_assets_impl(request)
}
