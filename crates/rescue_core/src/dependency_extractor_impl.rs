use crate::dependency_extractor::{
    DependencyExtractionError, DependencyExtractionMetadata, DependencyExtractionResult,
    DependencyExtractionWarning, DependencyRef, IgnoredInputSummary, DEPENDENCY_EXTRACTOR_VERSION,
    DEPENDENCY_REF_VERSION,
};
use crate::{ALSReadModel, ALSReadWarning, ActiveAudioReference};

const SUPPORTED_ALS_READ_MODEL_VERSION: &str = "0.2";

pub(crate) fn extract_dependencies_impl(model: &ALSReadModel) -> DependencyExtractionResult {
    let input_version = model.set_metadata.als_read_model_version.clone();
    if input_version != SUPPORTED_ALS_READ_MODEL_VERSION {
        return error_result(
            model,
            "DEPENDENCY_UNSUPPORTED_READ_MODEL_VERSION",
            format!(
                "unsupported ALSReadModel version {input_version}; expected {SUPPORTED_ALS_READ_MODEL_VERSION}"
            ),
        );
    }

    if !model.errors.is_empty() {
        return error_result(
            model,
            "DEPENDENCY_NO_TRUSTED_ALS_MODEL",
            "ALSReadModel contains fatal reader errors".to_string(),
        );
    }

    let mut dependencies = Vec::with_capacity(model.active_audio_references.len());
    let mut warnings = Vec::new();

    for (position, active_ref) in model.active_audio_references.iter().enumerate() {
        dependencies.push(make_dependency_ref(position, active_ref, &mut warnings));
    }

    let errors = Vec::new();
    DependencyExtractionResult {
        extraction_metadata: metadata(model, dependencies.len(), warnings.len(), errors.len()),
        dependencies,
        ignored_input_summary: ignored_input_summary(model),
        warnings,
        errors,
    }
}

fn make_dependency_ref(
    position: usize,
    active_ref: &ActiveAudioReference,
    warnings: &mut Vec<DependencyExtractionWarning>,
) -> DependencyRef {
    let dependency_id = format!("dep_audio_{position:06}");
    let (path_basis, extraction_status) =
        path_state(&active_ref.raw_path, &active_ref.raw_relative_path);
    let mut local_warnings = Vec::new();

    add_path_warnings(
        &dependency_id,
        active_ref.ref_id,
        &path_basis,
        &mut local_warnings,
        warnings,
    );
    add_missing_field_warnings(&dependency_id, active_ref, &mut local_warnings, warnings);
    add_upstream_warnings(&dependency_id, active_ref, &mut local_warnings, warnings);

    let evidence_status = if local_warnings.is_empty() {
        "extracted_from_als"
    } else {
        "extracted_with_warnings"
    };

    DependencyRef {
        dependency_id,
        dependency_kind: "audio_sample".to_string(),
        als_ref_id: active_ref.ref_id,
        source_kind: active_ref.source_kind.clone(),
        raw_path: active_ref.raw_path.clone(),
        raw_relative_path: active_ref.raw_relative_path.clone(),
        relative_path_type: active_ref.relative_path_type.clone(),
        file_type: active_ref.file_type.clone(),
        filename: active_ref.filename.clone(),
        extension: active_ref.extension.clone(),
        original_file_size: active_ref.original_file_size.clone(),
        original_crc: active_ref.original_crc.clone(),
        default_duration: active_ref.default_duration.clone(),
        default_sample_rate: active_ref.default_sample_rate.clone(),
        usage_context: active_ref.usage_context.clone(),
        xml_context: active_ref.xml_context.clone(),
        rewrite_support_status: active_ref.rewrite_support_status.clone(),
        extraction_status: extraction_status.to_string(),
        path_basis,
        evidence_status: evidence_status.to_string(),
        evidence_notes: evidence_notes(active_ref),
        warnings: local_warnings,
    }
}

fn path_state(
    raw_path: &Option<String>,
    raw_relative_path: &Option<String>,
) -> (String, &'static str) {
    match (has_text(raw_path), has_text(raw_relative_path)) {
        (true, true) => ("raw_path_and_raw_relative_path".to_string(), "extracted"),
        (true, false) => ("raw_path".to_string(), "extracted"),
        (false, true) => ("raw_relative_path".to_string(), "extracted"),
        (false, false) => ("path_unavailable".to_string(), "incomplete"),
    }
}

fn has_text(value: &Option<String>) -> bool {
    value.as_ref().is_some_and(|text| !text.trim().is_empty())
}

fn add_path_warnings(
    dependency_id: &str,
    als_ref_id: usize,
    path_basis: &str,
    local_warnings: &mut Vec<DependencyExtractionWarning>,
    warnings: &mut Vec<DependencyExtractionWarning>,
) {
    let warning = match path_basis {
        "raw_path" => Some((
            "DEPENDENCY_RAW_RELATIVE_PATH_MISSING",
            "raw_relative_path is missing; later path logic must be cautious",
        )),
        "raw_relative_path" => Some((
            "DEPENDENCY_RAW_PATH_MISSING",
            "raw_path is missing; later path logic must be cautious",
        )),
        "path_unavailable" => Some((
            "DEPENDENCY_PATH_UNAVAILABLE",
            "both raw_path and raw_relative_path are missing",
        )),
        _ => None,
    };

    if let Some((code, message)) = warning {
        push_warning(
            code,
            message,
            dependency_id,
            als_ref_id,
            local_warnings,
            warnings,
        );
    }
}

fn add_missing_field_warnings(
    dependency_id: &str,
    active_ref: &ActiveAudioReference,
    local_warnings: &mut Vec<DependencyExtractionWarning>,
    warnings: &mut Vec<DependencyExtractionWarning>,
) {
    if active_ref.relative_path_type.is_none() {
        push_warning(
            "DEPENDENCY_RELATIVE_PATH_TYPE_MISSING",
            "relative_path_type is missing",
            dependency_id,
            active_ref.ref_id,
            local_warnings,
            warnings,
        );
    }

    if active_ref.filename.is_none() {
        push_warning(
            "DEPENDENCY_FILENAME_MISSING",
            "filename is missing",
            dependency_id,
            active_ref.ref_id,
            local_warnings,
            warnings,
        );
    }
}

fn add_upstream_warnings(
    dependency_id: &str,
    active_ref: &ActiveAudioReference,
    local_warnings: &mut Vec<DependencyExtractionWarning>,
    warnings: &mut Vec<DependencyExtractionWarning>,
) {
    for upstream in &active_ref.warnings {
        let message = upstream_warning_message(upstream);
        push_warning(
            "DEPENDENCY_UPSTREAM_REF_WARNING_PROPAGATED",
            &message,
            dependency_id,
            active_ref.ref_id,
            local_warnings,
            warnings,
        );
    }
}

fn upstream_warning_message(upstream: &ALSReadWarning) -> String {
    format!(
        "upstream ALSReader warning {}: {}",
        upstream.warning_code, upstream.message
    )
}

fn push_warning(
    warning_code: &str,
    message: &str,
    dependency_id: &str,
    als_ref_id: usize,
    local_warnings: &mut Vec<DependencyExtractionWarning>,
    warnings: &mut Vec<DependencyExtractionWarning>,
) {
    let warning = DependencyExtractionWarning {
        warning_id: warnings.len(),
        warning_code: warning_code.to_string(),
        severity: "warning".to_string(),
        message: message.to_string(),
        dependency_id: Some(dependency_id.to_string()),
        als_ref_id: Some(als_ref_id),
        evidence_status: "extracted_with_warnings".to_string(),
    };
    warnings.push(warning.clone());
    local_warnings.push(warning);
}

fn evidence_notes(active_ref: &ActiveAudioReference) -> Vec<String> {
    let mut notes = Vec::new();
    if active_ref.original_file_size.is_some() {
        notes.push("original_file_size_supporting_signal".to_string());
    }
    if active_ref.original_crc.is_some() {
        notes.push("original_crc_weak_identity_signal".to_string());
    }
    notes
}

fn ignored_input_summary(model: &ALSReadModel) -> IgnoredInputSummary {
    IgnoredInputSummary {
        historical_refs_ignored_count: model.historical_refs.len(),
        non_audio_dependency_signals_ignored_count: model.non_audio_dependency_signals.len(),
        input_warnings_seen_count: model.warnings.len(),
        input_errors_seen_count: model.errors.len(),
        ignored_scope_notes: vec![
            "historical_refs_ignored_by_v0_1_scope".to_string(),
            "non_audio_signals_ignored_by_v0_1_scope".to_string(),
        ],
    }
}

fn error_result(
    model: &ALSReadModel,
    error_code: &str,
    message: String,
) -> DependencyExtractionResult {
    let errors = vec![DependencyExtractionError {
        error_code: error_code.to_string(),
        message,
        input_model_version: model.set_metadata.als_read_model_version.clone(),
    }];

    DependencyExtractionResult {
        extraction_metadata: metadata(model, 0, 0, errors.len()),
        dependencies: Vec::new(),
        ignored_input_summary: ignored_input_summary(model),
        warnings: Vec::new(),
        errors,
    }
}

fn metadata(
    model: &ALSReadModel,
    dependency_count: usize,
    warning_count: usize,
    error_count: usize,
) -> DependencyExtractionMetadata {
    DependencyExtractionMetadata {
        extractor_version: DEPENDENCY_EXTRACTOR_VERSION.to_string(),
        dependency_ref_version: DEPENDENCY_REF_VERSION.to_string(),
        input_als_read_model_version: model.set_metadata.als_read_model_version.clone(),
        source_als_path: model.set_metadata.source_als_path.clone(),
        source_project_root: model.set_metadata.source_project_root.clone(),
        source_file_hash: model.set_metadata.source_file_hash.clone(),
        dependency_count,
        warning_count,
        error_count,
    }
}
