use crate::{ResolutionCandidate, ResolutionEvidence};
use rescue_analyzer::RequiredAsset;
use rescue_catalog::FileOccurrence;
use std::path::Path;

pub(crate) fn scored_candidate(
    asset: &RequiredAsset,
    occurrence: &FileOccurrence,
) -> Option<ResolutionCandidate> {
    let exact_path = has_exact_observed_path(asset, &occurrence.native_path);
    let filename_match = filename_match(asset.filename.as_deref(), &occurrence.filename);
    if !exact_path && filename_match == FilenameMatch::None {
        return None;
    }

    let mut evidence = Vec::new();
    let mut conflicts = Vec::new();
    if exact_path {
        push_evidence(
            &mut evidence,
            "exact_observed_native_path",
            50,
            "Inventory path equals an accepted existing ALS path observation",
        );
    }
    match filename_match {
        FilenameMatch::Exact => push_evidence(
            &mut evidence,
            "exact_filename",
            20,
            "Filename equals the recorded filename",
        ),
        FilenameMatch::AsciiCaseInsensitive => push_evidence(
            &mut evidence,
            "ascii_case_insensitive_filename",
            10,
            "Filename differs only by ASCII case",
        ),
        FilenameMatch::None => conflicts.push("filename_differs".to_string()),
    }
    add_extension_evidence(asset, occurrence, &mut evidence);
    add_size_evidence(asset, occurrence, &mut evidence, &mut conflicts);
    let score = evidence.iter().map(|item| item.weight).sum();
    Some(ResolutionCandidate {
        candidate_id: format!(
            "{}:candidate:{}",
            asset.required_asset_id, occurrence.file_occurrence_id
        ),
        file_occurrence_id: occurrence.file_occurrence_id.clone(),
        content_id: occurrence.content_id.clone(),
        native_path: occurrence.native_path.clone(),
        score,
        evidence,
        conflicts,
    })
}

fn has_exact_observed_path(asset: &RequiredAsset, path: &Path) -> bool {
    asset.candidate_observations.iter().any(|candidate| {
        candidate.availability_status == "existing_regular_file"
            && candidate.safety_status == "safe_for_metadata_read"
            && Path::new(&candidate.candidate_path) == path
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FilenameMatch {
    Exact,
    AsciiCaseInsensitive,
    None,
}

fn filename_match(expected: Option<&str>, observed: &str) -> FilenameMatch {
    match expected {
        Some(value) if value == observed => FilenameMatch::Exact,
        Some(value) if value.eq_ignore_ascii_case(observed) => FilenameMatch::AsciiCaseInsensitive,
        _ => FilenameMatch::None,
    }
}

fn add_extension_evidence(
    asset: &RequiredAsset,
    occurrence: &FileOccurrence,
    evidence: &mut Vec<ResolutionEvidence>,
) {
    if asset
        .extension
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case(&occurrence.extension))
    {
        push_evidence(
            evidence,
            "exact_extension",
            5,
            "Extension equals the recorded extension",
        );
    }
}

fn add_size_evidence(
    asset: &RequiredAsset,
    occurrence: &FileOccurrence,
    evidence: &mut Vec<ResolutionEvidence>,
    conflicts: &mut Vec<String>,
) {
    let expected = asset
        .original_file_size
        .as_deref()
        .and_then(|value| value.parse::<u64>().ok());
    match expected {
        Some(size) if size == occurrence.file_size => push_evidence(
            evidence,
            "exact_expected_file_size",
            25,
            "Observed byte size equals the ALS-recorded size",
        ),
        Some(_) => conflicts.push("file_size_differs".to_string()),
        None => {}
    }
}

fn push_evidence(
    evidence: &mut Vec<ResolutionEvidence>,
    code: &str,
    weight: u8,
    explanation: &str,
) {
    evidence.push(ResolutionEvidence {
        evidence_code: code.to_string(),
        weight,
        explanation: explanation.to_string(),
    });
}
