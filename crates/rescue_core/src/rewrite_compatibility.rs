use crate::{ALSReadModel, ActiveAudioReference};
use serde::{Deserialize, Serialize};

pub const REWRITE_COMPATIBILITY_SCHEMA_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewriteCompatibilityAssessment {
    pub schema_version: String,
    pub document_profile: String,
    pub evidence_status: String,
    pub ableton_document_version: Option<String>,
    pub ableton_creator_version: Option<String>,
    pub ableton_minor_version: Option<String>,
    pub ableton_schema_change_count: Option<String>,
    pub active_reference_count: usize,
    pub strict_supported_count: usize,
    pub lab_compatible_count: usize,
    pub unsupported_shape_count: usize,
    pub references: Vec<RewriteReferenceCompatibility>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewriteReferenceCompatibility {
    pub ref_id: usize,
    pub usage_context: String,
    pub relative_path_type: Option<String>,
    pub strict_supported: bool,
    pub compatibility_lab_supported: bool,
    pub structure_profile: String,
    pub reason_codes: Vec<String>,
}

pub fn assess_rewrite_compatibility(model: &ALSReadModel) -> RewriteCompatibilityAssessment {
    let references: Vec<_> = model
        .active_audio_references
        .iter()
        .map(reference_compatibility)
        .collect();
    let strict_supported_count = references
        .iter()
        .filter(|reference| reference.strict_supported)
        .count();
    let lab_compatible_count = references
        .iter()
        .filter(|reference| reference.compatibility_lab_supported)
        .count();
    let unsupported_shape_count = references.len() - lab_compatible_count;
    let document_profile = if is_confirmed_live_11_3_document(model) {
        "confirmed_live_11_3"
    } else {
        "unconfirmed_ableton_version"
    };
    let evidence_status = if document_profile == "confirmed_live_11_3" {
        "confirmed_profile"
    } else if lab_compatible_count > 0 {
        "experimental_candidate"
    } else {
        "insufficient_structural_evidence"
    };

    RewriteCompatibilityAssessment {
        schema_version: REWRITE_COMPATIBILITY_SCHEMA_VERSION.to_string(),
        document_profile: document_profile.to_string(),
        evidence_status: evidence_status.to_string(),
        ableton_document_version: safe_metadata_value(
            model.set_metadata.ableton_document_version.as_deref(),
        ),
        ableton_creator_version: safe_metadata_value(
            model.set_metadata.ableton_creator_version.as_deref(),
        ),
        ableton_minor_version: safe_metadata_value(
            model.set_metadata.ableton_minor_version.as_deref(),
        ),
        ableton_schema_change_count: safe_metadata_value(
            model.set_metadata.ableton_schema_change_count.as_deref(),
        ),
        active_reference_count: references.len(),
        strict_supported_count,
        lab_compatible_count,
        unsupported_shape_count,
        references,
    }
}

pub fn reference_is_lab_compatible(reference: &ActiveAudioReference) -> bool {
    reference_compatibility(reference).compatibility_lab_supported
}

pub fn is_confirmed_live_11_3_document(model: &ALSReadModel) -> bool {
    model.set_metadata.ableton_document_version.as_deref() == Some("5")
        && model.set_metadata.ableton_minor_version.as_deref() == Some("11.0_11300")
        && model
            .set_metadata
            .ableton_creator_version
            .as_deref()
            .is_some_and(|value| value.starts_with("Ableton Live 11.3."))
}

fn reference_compatibility(reference: &ActiveAudioReference) -> RewriteReferenceCompatibility {
    let strict_supported =
        reference.is_rewrite_candidate && reference.rewrite_support_status == "supported";
    let (structure_profile, reason_codes) = structural_profile(reference);
    RewriteReferenceCompatibility {
        ref_id: reference.ref_id,
        usage_context: reference.usage_context.clone(),
        relative_path_type: reference.relative_path_type.clone(),
        strict_supported,
        compatibility_lab_supported: reason_codes.is_empty(),
        structure_profile: structure_profile.to_string(),
        reason_codes,
    }
}

fn structural_profile(reference: &ActiveAudioReference) -> (&'static str, Vec<String>) {
    let mut reasons = Vec::new();
    if !matches!(
        reference.usage_context.as_str(),
        "audio_clip" | "simpler_multisample"
    ) {
        reasons.push("unsupported_usage_context".to_string());
    }
    if reference.xml_locator != format!("SampleRef[{}]/FileRef", reference.ref_id) {
        reasons.push("unsupported_xml_locator".to_string());
    }
    let profile = match reference.relative_path_type.as_deref() {
        Some("1") => {
            if !reference.filename.as_deref().is_some_and(is_safe_filename) {
                reasons.push("invalid_or_missing_filename".to_string());
            }
            "external_type1"
        }
        Some("3") => {
            if !reference
                .raw_relative_path
                .as_deref()
                .is_some_and(is_safe_project_relative_path)
            {
                reasons.push("invalid_or_unsupported_project_relative_path".to_string());
            }
            "project_local_type3"
        }
        Some(_) => {
            reasons.push("unsupported_relative_path_type".to_string());
            "unsupported_path_type"
        }
        None => {
            reasons.push("missing_relative_path_type".to_string());
            "missing_path_type"
        }
    };
    (profile, reasons)
}

fn is_safe_filename(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && !matches!(trimmed, "." | "..")
        && !trimmed.contains('/')
        && !trimmed.contains('\\')
}

fn is_safe_project_relative_path(value: &str) -> bool {
    let normalized = value.replace('\\', "/");
    let components: Vec<_> = normalized.split('/').collect();
    components.len() >= 2
        && components.first() == Some(&"Samples")
        && components
            .iter()
            .all(|component| !component.is_empty() && !matches!(*component, "." | ".."))
}

fn safe_metadata_value(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    (!value.is_empty() && value.len() <= 120 && !value.contains(['/', '\\', '\n', '\r']))
        .then(|| value.to_string())
}
