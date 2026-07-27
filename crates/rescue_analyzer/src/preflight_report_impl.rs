use crate::{
    DependencyAssessmentResult, PreflightNotice, PreflightProjectContext, PreflightReport,
    PreflightReportError, PreflightReportMetadata, PreflightRequirement, PreflightSummary,
    ProjectDiscoveryResult, PREFLIGHT_REPORT_VERSION,
};
use std::collections::BTreeSet;

pub(crate) fn build_preflight_report_impl(
    discovery: &ProjectDiscoveryResult,
    assessment: &DependencyAssessmentResult,
) -> PreflightReport {
    if let Some((code, message)) = validate_handoff(discovery, assessment) {
        return blocked_report(discovery, assessment, code, message);
    }

    let requirements: Vec<_> = assessment
        .required_assets
        .iter()
        .map(|asset| PreflightRequirement {
            required_asset_id: asset.required_asset_id.clone(),
            filename: asset.filename.clone(),
            occurrence_count: asset.occurrence_count,
            availability_status: asset.availability_status.clone(),
            resolution_status: asset.resolution_status.clone(),
            candidate_paths: unique_candidate_paths(asset),
            risk_flags: asset.risk_flags.clone(),
        })
        .collect();
    let notices = notices(&requirements);
    PreflightReport {
        report_metadata: metadata(assessment, requirements.len(), notices.len(), 0),
        project: project_context(discovery),
        summary: summary(assessment),
        requirements,
        notices,
        errors: Vec::new(),
    }
}

fn validate_handoff(
    discovery: &ProjectDiscoveryResult,
    assessment: &DependencyAssessmentResult,
) -> Option<(&'static str, &'static str)> {
    if !discovery.errors.is_empty() {
        return Some((
            "PREFLIGHT_UNTRUSTED_PROJECT_DISCOVERY",
            "Project discovery contains fatal errors",
        ));
    }
    if !assessment.errors.is_empty() {
        return Some((
            "PREFLIGHT_UNTRUSTED_ASSESSMENT",
            "Dependency assessment contains fatal errors",
        ));
    }
    if discovery.metadata.source_als_path.to_string_lossy()
        != assessment.assessment_metadata.source_als_path
    {
        return Some((
            "PREFLIGHT_SOURCE_MISMATCH",
            "Project discovery and assessment describe different ALS paths",
        ));
    }
    None
}

fn unique_candidate_paths(asset: &crate::RequiredAsset) -> Vec<String> {
    asset
        .candidate_observations
        .iter()
        .map(|candidate| candidate.candidate_path.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn notices(requirements: &[PreflightRequirement]) -> Vec<PreflightNotice> {
    requirements
        .iter()
        .flat_map(|requirement| {
            requirement.risk_flags.iter().map(|risk| {
                (
                    risk.clone(),
                    format!("Requirement carries risk: {risk}"),
                    requirement.required_asset_id.clone(),
                )
            })
        })
        .enumerate()
        .map(
            |(notice_id, (notice_code, message, required_asset_id))| PreflightNotice {
                notice_id,
                notice_code,
                message,
                required_asset_id: Some(required_asset_id),
            },
        )
        .collect()
}

fn summary(assessment: &DependencyAssessmentResult) -> PreflightSummary {
    let metadata = &assessment.assessment_metadata;
    let overall_status = if metadata.missing_candidate_asset_count > 0 {
        "needs_asset_search"
    } else if metadata.unknown_asset_count > 0 || metadata.required_asset_count == 0 {
        "needs_review"
    } else {
        "candidates_observed_not_resolved"
    };
    PreflightSummary {
        overall_status: overall_status.to_string(),
        reference_occurrence_count: metadata.occurrence_count,
        required_asset_count: metadata.required_asset_count,
        candidate_observed_count: metadata.regular_file_candidate_asset_count,
        needs_search_count: metadata.missing_candidate_asset_count,
        unknown_count: metadata.unknown_asset_count,
        unresolved_count: metadata.required_asset_count,
    }
}

fn project_context(discovery: &ProjectDiscoveryResult) -> PreflightProjectContext {
    PreflightProjectContext {
        discovery_status: discovery.discovery_status.clone(),
        confirmed_project_root: discovery
            .confirmed_project_root
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        set_location: discovery.set_location.clone(),
        candidate_root_count: discovery.candidates.len(),
    }
}

fn metadata(
    assessment: &DependencyAssessmentResult,
    requirement_count: usize,
    notice_count: usize,
    error_count: usize,
) -> PreflightReportMetadata {
    PreflightReportMetadata {
        report_version: PREFLIGHT_REPORT_VERSION.to_string(),
        assessment_version: assessment.assessment_metadata.assessment_version.clone(),
        source_als_path: assessment.assessment_metadata.source_als_path.clone(),
        source_file_hash: assessment.assessment_metadata.source_file_hash.clone(),
        requirement_count,
        notice_count,
        error_count,
    }
}

fn blocked_report(
    discovery: &ProjectDiscoveryResult,
    assessment: &DependencyAssessmentResult,
    code: &str,
    message: &str,
) -> PreflightReport {
    PreflightReport {
        report_metadata: metadata(assessment, 0, 0, 1),
        project: project_context(discovery),
        summary: PreflightSummary {
            overall_status: "blocked".to_string(),
            reference_occurrence_count: 0,
            required_asset_count: 0,
            candidate_observed_count: 0,
            needs_search_count: 0,
            unknown_count: 0,
            unresolved_count: 0,
        },
        requirements: Vec::new(),
        notices: Vec::new(),
        errors: vec![PreflightReportError {
            error_code: code.to_string(),
            message: message.to_string(),
        }],
    }
}
