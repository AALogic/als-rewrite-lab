use rescue_core::DependencyRef;

use crate::RequiredAssetCandidateObservation;

pub(crate) const CORE_LIBRARY_CATEGORY: &str = "ableton_core_library";
pub(crate) const SYSTEM_DEPENDENCY_CLASS: &str = "system_dependency";

pub(crate) struct SourceClassification {
    pub source_category: &'static str,
    pub management_class: &'static str,
    pub status: &'static str,
    pub basis: &'static str,
}

pub(crate) fn classify(
    dependency: &DependencyRef,
    candidates: &[RequiredAssetCandidateObservation],
) -> SourceClassification {
    if confirmed_macos_core_library(dependency, candidates) {
        SourceClassification {
            source_category: CORE_LIBRARY_CATEGORY,
            management_class: SYSTEM_DEPENDENCY_CLASS,
            status: "confirmed",
            basis: "macos_core_library_path_relative_type_5_and_regular_file",
        }
    } else {
        SourceClassification {
            source_category: "unclassified",
            management_class: "unclassified",
            status: "unknown",
            basis: "insufficient_source_category_evidence",
        }
    }
}

fn confirmed_macos_core_library(
    dependency: &DependencyRef,
    candidates: &[RequiredAssetCandidateObservation],
) -> bool {
    dependency.relative_path_type.as_deref() == Some("5")
        && dependency
            .raw_relative_path
            .as_deref()
            .is_some_and(safe_samples_relative_path)
        && dependency
            .raw_path
            .as_deref()
            .is_some_and(macos_core_library_path)
        && candidates.iter().any(|candidate| {
            dependency.raw_path.as_deref() == Some(candidate.candidate_path.as_str())
                && candidate.candidate_basis == "recorded_raw_absolute_path"
                && candidate.platform_status == "checkable_on_current_platform"
                && candidate.safety_status == "safe_for_metadata_read"
                && candidate.availability_status == "existing_regular_file"
                && candidate.entry_kind == "regular_file"
                && candidate.size_evidence_status != "differs_from_expected_size"
        })
}

fn macos_core_library_path(raw: &str) -> bool {
    let normalized = raw.replace('\\', "/");
    let Some(remainder) = normalized.strip_prefix("/Applications/") else {
        return false;
    };
    let components: Vec<_> = remainder.split('/').collect();
    components.len() >= 5
        && components[0].starts_with("Ableton Live ")
        && components[0].ends_with(".app")
        && components[1] == "Contents"
        && components[2] == "App-Resources"
        && components[3] == "Core Library"
        && components[4..]
            .iter()
            .all(|component| !component.is_empty() && !matches!(*component, "." | ".."))
}

fn safe_samples_relative_path(raw: &str) -> bool {
    let normalized = raw.replace('\\', "/");
    let components: Vec<_> = normalized.split('/').collect();
    components.len() >= 2
        && components[0] == "Samples"
        && components
            .iter()
            .all(|component| !component.is_empty() && !matches!(*component, "." | ".."))
}

#[cfg(test)]
mod tests {
    use super::{classify, CORE_LIBRARY_CATEGORY, SYSTEM_DEPENDENCY_CLASS};
    use crate::RequiredAssetCandidateObservation;
    use rescue_core::DependencyRef;

    fn dependency(path: &str, relative_type: &str) -> DependencyRef {
        DependencyRef {
            dependency_id: "dependency_000000".to_string(),
            dependency_kind: "audio_sample".to_string(),
            als_ref_id: 0,
            source_kind: "sample_ref_file_ref".to_string(),
            raw_path: Some(path.to_string()),
            raw_relative_path: Some("Samples/Multisamples/Drums/Kick.wav".to_string()),
            relative_path_type: Some(relative_type.to_string()),
            file_type: Some("2".to_string()),
            filename: Some("Kick.wav".to_string()),
            extension: Some("wav".to_string()),
            original_file_size: Some("100".to_string()),
            original_crc: Some("123".to_string()),
            default_duration: None,
            default_sample_rate: None,
            usage_context: "simpler_multisample".to_string(),
            xml_context: "SampleRef/FileRef".to_string(),
            rewrite_support_status: "requires_test".to_string(),
            extraction_status: "active_audio_dependency".to_string(),
            path_basis: "raw_and_relative_path".to_string(),
            evidence_status: "observed".to_string(),
            evidence_notes: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn candidate(path: &str) -> RequiredAssetCandidateObservation {
        RequiredAssetCandidateObservation {
            dependency_id: "dependency_000000".to_string(),
            als_ref_id: 0,
            candidate_id: "candidate_000000".to_string(),
            candidate_basis: "recorded_raw_absolute_path".to_string(),
            candidate_path: path.to_string(),
            platform_status: "checkable_on_current_platform".to_string(),
            safety_status: "safe_for_metadata_read".to_string(),
            availability_status: "existing_regular_file".to_string(),
            entry_kind: "regular_file".to_string(),
            size_evidence_status: "matches_expected_size".to_string(),
            observed_file_size: Some(100),
            expected_file_size: Some(100),
        }
    }

    #[test]
    fn confirmed_macos_core_library_reference_is_system_dependency() {
        let path = "/Applications/Ableton Live 11 Suite.app/Contents/App-Resources/Core Library/Samples/Multisamples/Drums/Kick.wav";
        let result = classify(&dependency(path, "5"), &[candidate(path)]);

        assert_eq!(result.source_category, CORE_LIBRARY_CATEGORY);
        assert_eq!(result.management_class, SYSTEM_DEPENDENCY_CLASS);
        assert_eq!(result.status, "confirmed");
    }

    #[test]
    fn relative_type_5_outside_core_library_is_not_silently_system_managed() {
        let path = "/Users/test/Core Library/Samples/Multisamples/Drums/Kick.wav";
        let result = classify(&dependency(path, "5"), &[candidate(path)]);

        assert_eq!(result.source_category, "unclassified");
        assert_eq!(result.management_class, "unclassified");
        assert_eq!(result.status, "unknown");
    }

    #[test]
    fn core_library_path_without_relative_type_5_is_not_system_dependency() {
        let path = "/Applications/Ableton Live 11 Suite.app/Contents/App-Resources/Core Library/Samples/Multisamples/Drums/Kick.wav";
        let result = classify(&dependency(path, "1"), &[candidate(path)]);

        assert_eq!(result.management_class, "unclassified");
    }
}
