use crate::{DependencyAssessmentResult, ProjectDiscoveryResult};
use serde::{Deserialize, Serialize};

pub const PREFLIGHT_REPORT_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightReport {
    pub report_metadata: PreflightReportMetadata,
    pub project: PreflightProjectContext,
    pub summary: PreflightSummary,
    pub requirements: Vec<PreflightRequirement>,
    pub notices: Vec<PreflightNotice>,
    pub errors: Vec<PreflightReportError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightReportMetadata {
    pub report_version: String,
    pub assessment_version: String,
    pub source_als_path: String,
    pub source_file_hash: String,
    pub requirement_count: usize,
    pub notice_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightProjectContext {
    pub discovery_status: String,
    pub confirmed_project_root: Option<String>,
    pub set_location: String,
    pub candidate_root_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightSummary {
    pub overall_status: String,
    pub reference_occurrence_count: usize,
    pub required_asset_count: usize,
    pub candidate_observed_count: usize,
    pub needs_search_count: usize,
    pub unknown_count: usize,
    pub unresolved_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightRequirement {
    pub required_asset_id: String,
    pub filename: Option<String>,
    pub occurrence_count: usize,
    pub availability_status: String,
    pub resolution_status: String,
    pub candidate_paths: Vec<String>,
    pub risk_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightNotice {
    pub notice_id: usize,
    pub notice_code: String,
    pub message: String,
    pub required_asset_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightReportError {
    pub error_code: String,
    pub message: String,
}

pub fn build_preflight_report(
    discovery: &ProjectDiscoveryResult,
    assessment: &DependencyAssessmentResult,
) -> PreflightReport {
    crate::preflight_report_impl::build_preflight_report_impl(discovery, assessment)
}
