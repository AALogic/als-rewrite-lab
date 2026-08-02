mod dependency_assessment;
mod dependency_assessment_grouping;
mod dependency_assessment_impl;
mod dependency_assessment_result;
mod dependency_assessment_status;
mod dependency_source_classification;
mod preflight_report;
mod preflight_report_impl;
mod project_discovery;
mod project_discovery_impl;

pub use dependency_assessment::{
    assess_dependencies, DependencyAssessmentError, DependencyAssessmentMetadata,
    DependencyAssessmentResult, DependencyAssessmentWarning, RequiredAsset,
    RequiredAssetCandidateObservation, DEPENDENCY_ASSESSMENT_VERSION,
};
pub use preflight_report::{
    build_preflight_report, PreflightNotice, PreflightProjectContext, PreflightReport,
    PreflightReportError, PreflightReportMetadata, PreflightRequirement, PreflightSummary,
    PREFLIGHT_REPORT_VERSION,
};
pub use project_discovery::{
    discover_project, ProjectDiscoveryError, ProjectDiscoveryMetadata, ProjectDiscoveryRequest,
    ProjectDiscoveryResult, ProjectDiscoveryWarning, ProjectRootCandidate,
    PROJECT_DISCOVERY_VERSION,
};
