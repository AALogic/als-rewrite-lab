use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ALSReadModel {
    pub set_metadata: SetMetadata,
    pub active_audio_references: Vec<ActiveAudioReference>,
    pub historical_refs: Vec<HistoricalReference>,
    pub non_audio_dependency_signals: Vec<NonAudioDependencySignal>,
    pub warnings: Vec<ALSReadWarning>,
    pub errors: Vec<ALSReadError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetMetadata {
    pub source_als_path: String,
    pub source_als_filename: Option<String>,
    /// Compatibility field for ALSReadModel v0.2. ALSReader never infers this value.
    pub source_project_root: Option<String>,
    pub source_file_size: u64,
    pub source_file_hash: String,
    pub analysis_started_at: String,
    pub analysis_completed_at: String,
    pub reader_version: String,
    pub als_read_model_version: String,
    pub ableton_document_version: Option<String>,
    pub ableton_creator_version: Option<String>,
    pub ableton_minor_version: Option<String>,
    pub ableton_schema_change_count: Option<String>,
    pub decompressed_xml_size: usize,
    pub xml_root_name: String,
    pub sample_ref_count: usize,
    pub active_audio_ref_count: usize,
    pub historical_ref_count: usize,
    pub non_audio_signal_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveAudioReference {
    pub ref_id: usize,
    pub source_kind: String,
    pub raw_path: Option<String>,
    pub raw_relative_path: Option<String>,
    pub relative_path_type: Option<String>,
    pub file_type: Option<String>,
    pub filename: Option<String>,
    pub extension: Option<String>,
    pub original_file_size: Option<String>,
    pub original_crc: Option<String>,
    pub default_duration: Option<String>,
    pub default_sample_rate: Option<String>,
    pub usage_context: String,
    pub xml_context: String,
    pub xml_locator: String,
    pub is_rewrite_candidate: bool,
    pub rewrite_support_status: String,
    pub warnings: Vec<ALSReadWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalReference {
    pub ref_id: usize,
    pub source_kind: String,
    pub raw_path: Option<String>,
    pub raw_relative_path: Option<String>,
    pub relative_path_type: Option<String>,
    pub filename: Option<String>,
    pub extension: Option<String>,
    pub original_file_size: Option<String>,
    pub original_crc: Option<String>,
    pub xml_context: String,
    pub xml_locator: String,
    pub linked_active_ref_id: Option<usize>,
    pub usage_note: String,
    pub warnings: Vec<ALSReadWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonAudioDependencySignal {
    pub signal_id: usize,
    pub signal_kind: String,
    pub name: Option<String>,
    pub raw_path: Option<String>,
    pub raw_relative_path: Option<String>,
    pub relative_path_type: Option<String>,
    pub plugin_format: Option<String>,
    pub plugin_identifier: Option<String>,
    pub plugin_version: Option<String>,
    pub xml_context: String,
    pub xml_locator: String,
    pub support_status: String,
    pub warnings: Vec<ALSReadWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ALSReadWarning {
    pub warning_id: usize,
    pub warning_code: String,
    pub severity: String,
    pub message: String,
    pub xml_context: Option<String>,
    pub related_ref_id: Option<usize>,
    pub evidence_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ALSReadError {
    pub error_code: String,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Debug, Error)]
pub enum ALSError {
    #[error("file not found: {path}")]
    FileNotFound { path: String },
    #[error("file not readable: {path}: {message}")]
    FileNotReadable { path: String, message: String },
    #[error("ALS file is too large: {path}: {size} bytes exceeds {limit} bytes")]
    CompressedTooLarge { path: String, size: u64, limit: u64 },
    #[error("decompressed ALS XML is too large: {path}: limit {limit} bytes exceeded")]
    DecompressedXmlTooLarge { path: String, limit: u64 },
    #[error("not a gzip-compressed ALS file: {path}")]
    NotGzip { path: String },
    #[error("invalid XML in ALS file: {path}: {message}")]
    InvalidXml { path: String, message: String },
    #[error("missing Ableton root element: {path}")]
    MissingAbletonRoot { path: String },
    #[error("unsupported internal error: {message}")]
    UnsupportedInternalError { message: String },
}

impl ALSError {
    pub fn to_info(&self) -> ALSReadError {
        match self {
            Self::FileNotFound { path } => ALSReadError {
                error_code: "ALS_NOT_FOUND".to_string(),
                message: self.to_string(),
                path: Some(path.clone()),
            },
            Self::FileNotReadable { path, .. } => ALSReadError {
                error_code: "ALS_NOT_READABLE".to_string(),
                message: self.to_string(),
                path: Some(path.clone()),
            },
            Self::CompressedTooLarge { path, .. } => ALSReadError {
                error_code: "ALS_COMPRESSED_TOO_LARGE".to_string(),
                message: self.to_string(),
                path: Some(path.clone()),
            },
            Self::DecompressedXmlTooLarge { path, .. } => ALSReadError {
                error_code: "ALS_DECOMPRESSED_XML_TOO_LARGE".to_string(),
                message: self.to_string(),
                path: Some(path.clone()),
            },
            Self::NotGzip { path } => ALSReadError {
                error_code: "ALS_NOT_GZIP".to_string(),
                message: self.to_string(),
                path: Some(path.clone()),
            },
            Self::InvalidXml { path, .. } => ALSReadError {
                error_code: "ALS_XML_INVALID".to_string(),
                message: self.to_string(),
                path: Some(path.clone()),
            },
            Self::MissingAbletonRoot { path } => ALSReadError {
                error_code: "ALS_UNSUPPORTED_ROOT".to_string(),
                message: self.to_string(),
                path: Some(path.clone()),
            },
            Self::UnsupportedInternalError { .. } => ALSReadError {
                error_code: "ALS_INTERNAL_ERROR".to_string(),
                message: self.to_string(),
                path: None,
            },
        }
    }
}
