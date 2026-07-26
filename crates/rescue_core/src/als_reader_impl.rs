use crate::{
    ALSError, ALSReadModel, ALSReadWarning, ActiveAudioReference, HistoricalReference,
    NonAudioDependencySignal, SetMetadata,
};
use std::path::Path;

use crate::als_reader::{ALS_READER_VERSION, ALS_READ_MODEL_VERSION};
use crate::als_reader_io::{decode_als, unix_timestamp_millis};
use crate::path_text::{extension_from_candidates, filename_from_candidates};

pub(crate) fn analyze_als_impl(path: &Path) -> Result<ALSReadModel, ALSError> {
    let analysis_started_at = unix_timestamp_millis();
    let decoded = decode_als(path)?;
    let document =
        roxmltree::Document::parse(&decoded.xml).map_err(|error| ALSError::InvalidXml {
            path: decoded.source_path.clone(),
            message: error.to_string(),
        })?;

    let root = document.root_element();
    if !root.has_tag_name("Ableton") {
        return Err(ALSError::MissingAbletonRoot {
            path: decoded.source_path,
        });
    }

    let mut warnings = Vec::new();
    let mut next_warning_id = 0;

    let (active_audio_references, sample_ref_count) =
        extract_active_audio_references(&document, &mut warnings, &mut next_warning_id);
    let historical_refs = extract_historical_refs(&document);
    let non_audio_dependency_signals = extract_non_audio_dependency_signals(&document);

    let analysis_completed_at = unix_timestamp_millis();
    let counts = AnalysisCounts {
        sample_ref: sample_ref_count,
        active_audio_ref: active_audio_references.len(),
        historical_ref: historical_refs.len(),
        non_audio_signal: non_audio_dependency_signals.len(),
        warning: warnings.len(),
    };
    let set_metadata = build_set_metadata(
        path,
        root,
        decoded.source_path,
        decoded.source_file_size,
        decoded.source_file_hash,
        analysis_started_at,
        analysis_completed_at,
        decoded.xml.len(),
        counts,
    );

    Ok(ALSReadModel {
        set_metadata,
        active_audio_references,
        historical_refs,
        non_audio_dependency_signals,
        warnings,
        errors: Vec::new(),
    })
}

struct AnalysisCounts {
    sample_ref: usize,
    active_audio_ref: usize,
    historical_ref: usize,
    non_audio_signal: usize,
    warning: usize,
}

#[allow(clippy::too_many_arguments)]
fn build_set_metadata(
    path: &Path,
    root: roxmltree::Node<'_, '_>,
    source_als_path: String,
    source_file_size: u64,
    source_file_hash: String,
    analysis_started_at: String,
    analysis_completed_at: String,
    decompressed_xml_size: usize,
    counts: AnalysisCounts,
) -> SetMetadata {
    SetMetadata {
        source_als_path,
        source_als_filename: path
            .file_name()
            .map(|value| value.to_string_lossy().to_string()),
        // Kept null for v0.2 compatibility. Project root belongs to discovery/context.
        source_project_root: None,
        source_file_size,
        source_file_hash,
        analysis_started_at,
        analysis_completed_at,
        reader_version: ALS_READER_VERSION.to_string(),
        als_read_model_version: ALS_READ_MODEL_VERSION.to_string(),
        ableton_document_version: root.attribute("MajorVersion").map(ToString::to_string),
        ableton_creator_version: root.attribute("Creator").map(ToString::to_string),
        ableton_minor_version: root.attribute("MinorVersion").map(ToString::to_string),
        ableton_schema_change_count: root.attribute("SchemaChangeCount").map(ToString::to_string),
        decompressed_xml_size,
        xml_root_name: root.tag_name().name().to_string(),
        sample_ref_count: counts.sample_ref,
        active_audio_ref_count: counts.active_audio_ref,
        historical_ref_count: counts.historical_ref,
        non_audio_signal_count: counts.non_audio_signal,
        warning_count: counts.warning,
        error_count: 0,
    }
}

fn extract_active_audio_references(
    document: &roxmltree::Document<'_>,
    warnings: &mut Vec<ALSReadWarning>,
    next_warning_id: &mut usize,
) -> (Vec<ActiveAudioReference>, usize) {
    let sample_refs: Vec<_> = document
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("SampleRef"))
        .collect();
    let mut active_audio_references = Vec::new();

    for (sample_index, sample_ref) in sample_refs.iter().enumerate() {
        let Some(file_ref) = direct_child(*sample_ref, "FileRef") else {
            warnings.push(read_warning(
                next_warning_id,
                "sample_ref_without_file_ref",
                "warning",
                "SampleRef has no direct FileRef child",
                Some("SampleRef"),
                Some(sample_index),
                "confirmed",
            ));
            continue;
        };

        let raw_path = child_value(file_ref, "Path");
        let raw_relative_path = child_value(file_ref, "RelativePath");
        let relative_path_type = child_value(file_ref, "RelativePathType");
        let mut ref_warnings = Vec::new();

        if raw_path.as_deref().unwrap_or("").is_empty() {
            let warning = read_warning(
                next_warning_id,
                "empty_path",
                "warning",
                "Active FileRef has empty Path",
                Some("SampleRef/FileRef"),
                Some(sample_index),
                "confirmed",
            );
            warnings.push(warning.clone());
            ref_warnings.push(warning);
        }

        match relative_path_type.as_deref() {
            Some("0" | "1" | "3" | "5" | "6") => {}
            Some(value) => {
                let warning = read_warning(
                    next_warning_id,
                    "unknown_relative_path_type",
                    "risk",
                    &format!("Unknown RelativePathType: {value}"),
                    Some("SampleRef/FileRef/RelativePathType"),
                    Some(sample_index),
                    "unknown",
                );
                warnings.push(warning.clone());
                ref_warnings.push(warning);
            }
            None => {
                let warning = read_warning(
                    next_warning_id,
                    "missing_relative_path_type",
                    "warning",
                    "Active FileRef has no RelativePathType",
                    Some("SampleRef/FileRef"),
                    Some(sample_index),
                    "confirmed",
                );
                warnings.push(warning.clone());
                ref_warnings.push(warning);
            }
        }

        let filename = filename_from(raw_path.as_deref(), raw_relative_path.as_deref());
        let extension = extension_from(raw_path.as_deref(), raw_relative_path.as_deref());

        active_audio_references.push(ActiveAudioReference {
            ref_id: sample_index,
            source_kind: "sample_ref_file_ref".to_string(),
            raw_path,
            raw_relative_path,
            relative_path_type,
            file_type: child_value(file_ref, "Type"),
            filename,
            extension,
            original_file_size: child_value(file_ref, "OriginalFileSize"),
            original_crc: child_value(file_ref, "OriginalCrc"),
            default_duration: child_value(*sample_ref, "DefaultDuration"),
            default_sample_rate: child_value(*sample_ref, "DefaultSampleRate"),
            usage_context: "unknown".to_string(),
            xml_context: "SampleRef/FileRef".to_string(),
            xml_locator: format!("SampleRef[{sample_index}]/FileRef"),
            is_rewrite_candidate: false,
            rewrite_support_status: "requires_test".to_string(),
            warnings: ref_warnings,
        });
    }

    (active_audio_references, sample_refs.len())
}

fn extract_historical_refs(document: &roxmltree::Document<'_>) -> Vec<HistoricalReference> {
    let mut historical_refs = Vec::new();

    for (historical_index, original_ref) in document
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("OriginalFileRef"))
        .enumerate()
    {
        let value_node = direct_child(original_ref, "FileRef").unwrap_or(original_ref);
        let raw_path = child_value(value_node, "Path");
        let raw_relative_path = child_value(value_node, "RelativePath");
        let relative_path_type = child_value(value_node, "RelativePathType");
        let filename = filename_from(raw_path.as_deref(), raw_relative_path.as_deref());
        let extension = extension_from(raw_path.as_deref(), raw_relative_path.as_deref());

        historical_refs.push(HistoricalReference {
            ref_id: historical_index,
            source_kind: "original_file_ref".to_string(),
            raw_path,
            raw_relative_path,
            relative_path_type,
            filename,
            extension,
            original_file_size: child_value(value_node, "OriginalFileSize"),
            original_crc: child_value(value_node, "OriginalCrc"),
            xml_context: xml_context_for(original_ref),
            xml_locator: format!("OriginalFileRef[{historical_index}]"),
            linked_active_ref_id: None,
            usage_note: "historical_provenance".to_string(),
            warnings: Vec::new(),
        });
    }

    historical_refs
}

fn extract_non_audio_dependency_signals(
    document: &roxmltree::Document<'_>,
) -> Vec<NonAudioDependencySignal> {
    let mut non_audio_dependency_signals = Vec::new();

    for (signal_index, file_ref) in document
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("FileRef"))
        .filter(|node| !is_direct_child_of(*node, "SampleRef"))
        .filter(|node| !has_ancestor(*node, "OriginalFileRef"))
        .enumerate()
    {
        let raw_path = child_value(file_ref, "Path");
        let raw_relative_path = child_value(file_ref, "RelativePath");

        non_audio_dependency_signals.push(NonAudioDependencySignal {
            signal_id: signal_index,
            signal_kind: "unknown_file_ref".to_string(),
            name: filename_from(raw_path.as_deref(), raw_relative_path.as_deref()),
            raw_path,
            raw_relative_path,
            relative_path_type: child_value(file_ref, "RelativePathType"),
            plugin_format: None,
            plugin_identifier: None,
            plugin_version: None,
            xml_context: xml_context_for(file_ref),
            xml_locator: format!("FileRef(non_audio)[{signal_index}]"),
            support_status: "report_only".to_string(),
            warnings: Vec::new(),
        });
    }

    non_audio_dependency_signals
}

fn direct_child<'a, 'input>(
    node: roxmltree::Node<'a, 'input>,
    tag_name: &str,
) -> Option<roxmltree::Node<'a, 'input>> {
    node.children()
        .find(|child| child.is_element() && child.tag_name().name() == tag_name)
}

fn child_value(node: roxmltree::Node<'_, '_>, tag_name: &str) -> Option<String> {
    direct_child(node, tag_name)
        .and_then(|child| child.attribute("Value"))
        .map(ToString::to_string)
}

fn filename_from(path: Option<&str>, relative_path: Option<&str>) -> Option<String> {
    filename_from_candidates(path, relative_path)
}

fn extension_from(path: Option<&str>, relative_path: Option<&str>) -> Option<String> {
    extension_from_candidates(path, relative_path)
}

fn has_ancestor(node: roxmltree::Node<'_, '_>, tag_name: &str) -> bool {
    node.ancestors()
        .any(|ancestor| ancestor.is_element() && ancestor.has_tag_name(tag_name))
}

fn is_direct_child_of(node: roxmltree::Node<'_, '_>, parent_tag_name: &str) -> bool {
    node.parent()
        .filter(|parent| parent.is_element() && parent.has_tag_name(parent_tag_name))
        .is_some()
}

fn xml_context_for(node: roxmltree::Node<'_, '_>) -> String {
    let mut names: Vec<_> = node
        .ancestors()
        .filter(|ancestor| ancestor.is_element())
        .map(|ancestor| ancestor.tag_name().name().to_string())
        .collect();
    names.reverse();
    names.join("/")
}

fn read_warning(
    next_warning_id: &mut usize,
    warning_code: &str,
    severity: &str,
    message: &str,
    xml_context: Option<&str>,
    related_ref_id: Option<usize>,
    evidence_status: &str,
) -> ALSReadWarning {
    let warning = ALSReadWarning {
        warning_id: *next_warning_id,
        warning_code: warning_code.to_string(),
        severity: severity.to_string(),
        message: message.to_string(),
        xml_context: xml_context.map(ToString::to_string),
        related_ref_id,
        evidence_status: evidence_status.to_string(),
    };
    *next_warning_id += 1;
    warning
}
