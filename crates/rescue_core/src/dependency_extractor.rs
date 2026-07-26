use crate::ALSReadModel;
use serde::{Deserialize, Serialize};

pub const DEPENDENCY_EXTRACTOR_VERSION: &str = "0.1";
pub const DEPENDENCY_REF_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyExtractionResult {
    pub extraction_metadata: DependencyExtractionMetadata,
    pub dependencies: Vec<DependencyRef>,
    pub ignored_input_summary: IgnoredInputSummary,
    pub warnings: Vec<DependencyExtractionWarning>,
    pub errors: Vec<DependencyExtractionError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyExtractionMetadata {
    pub extractor_version: String,
    pub dependency_ref_version: String,
    pub input_als_read_model_version: String,
    pub source_als_path: String,
    /// Compatibility field copied from ALSReadModel v0.2; not a discovered root.
    pub source_project_root: Option<String>,
    pub source_file_hash: String,
    pub dependency_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyRef {
    pub dependency_id: String,
    pub dependency_kind: String,
    pub als_ref_id: usize,
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
    pub rewrite_support_status: String,
    pub extraction_status: String,
    pub path_basis: String,
    pub evidence_status: String,
    pub evidence_notes: Vec<String>,
    pub warnings: Vec<DependencyExtractionWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IgnoredInputSummary {
    pub historical_refs_ignored_count: usize,
    pub non_audio_dependency_signals_ignored_count: usize,
    pub input_warnings_seen_count: usize,
    pub input_errors_seen_count: usize,
    pub ignored_scope_notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyExtractionWarning {
    pub warning_id: usize,
    pub warning_code: String,
    pub severity: String,
    pub message: String,
    pub dependency_id: Option<String>,
    pub als_ref_id: Option<usize>,
    pub evidence_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyExtractionError {
    pub error_code: String,
    pub message: String,
    pub input_model_version: String,
}

pub fn extract_dependencies(model: &ALSReadModel) -> DependencyExtractionResult {
    crate::dependency_extractor_impl::extract_dependencies_impl(model)
}
