use crate::{PackageValidationError, SemanticDiffRecord};
use flate2::read::GzDecoder;
use rescue_core::{MAX_COMPRESSED_ALS_BYTES, MAX_DECOMPRESSED_XML_BYTES};
use rescue_packaging::{PackagePlan, RewriteOperation};
use rescue_rewriter::ALSRewriteResult;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::ops::Range;
use std::path::Path;

pub(crate) struct SemanticDiffOutcome {
    pub records: Vec<SemanticDiffRecord>,
    pub errors: Vec<PackageValidationError>,
}

struct DecodedAls {
    hash: String,
    xml: String,
}

struct Mask {
    range: Range<usize>,
    token: String,
}

pub(crate) fn validate_semantic_diff(
    plan: &PackagePlan,
    staged_als: &Path,
    rewrite: &ALSRewriteResult,
) -> SemanticDiffOutcome {
    let original = match decode(&plan.source_als.source_als_path) {
        Ok(decoded) => decoded,
        Err(error) => return failed(error),
    };
    if original.hash != plan.source_als.source_file_hash {
        return failed(error(
            "ORIGINAL_FILE_CHANGED",
            "Original ALS no longer matches the planned source snapshot",
            None,
            Some(&plan.source_als.source_als_path),
        ));
    }
    let rewritten = match decode(staged_als) {
        Ok(decoded) => decoded,
        Err(error) => return failed(error),
    };
    if rewrite.rewritten_staged_als_hash.as_deref() != Some(rewritten.hash.as_str()) {
        return failed(error(
            "VALIDATION_REWRITTEN_HASH_MISMATCH",
            "Rewritten ALS hash does not match the rewrite result",
            None,
            Some(staged_als),
        ));
    }
    compare_xml(&original.xml, &rewritten.xml, &plan.rewrite_operations)
}

fn compare_xml(
    original: &str,
    rewritten: &str,
    operations: &[RewriteOperation],
) -> SemanticDiffOutcome {
    let original_masks = match locate_masks(original, operations, false) {
        Ok(masks) => masks,
        Err(error) => return failed(error),
    };
    let rewritten_masks = match locate_masks(rewritten, operations, true) {
        Ok(masks) => masks,
        Err(error) => return failed(error),
    };
    let original_masked = apply_masks(original, original_masks);
    let rewritten_masked = apply_masks(rewritten, rewritten_masks);
    if original_masked != rewritten_masked {
        return failed(error(
            "SEMANTIC_DIFF_UNEXPECTED_CHANGE",
            "ALS XML changed outside the approved attribute values",
            None,
            None,
        ));
    }
    SemanticDiffOutcome {
        records: operations
            .iter()
            .map(|operation| SemanticDiffRecord {
                operation_id: operation.operation_id.clone(),
                als_ref_id: operation.als_ref_id,
                xml_locator: operation.xml_locator.clone(),
                verified_fields: operation.fields_to_change.clone(),
                diff_status: "verified_allowed_change".to_string(),
            })
            .collect(),
        errors: Vec::new(),
    }
}

fn locate_masks(
    xml: &str,
    operations: &[RewriteOperation],
    use_new_values: bool,
) -> Result<Vec<Mask>, PackageValidationError> {
    let document = roxmltree::Document::parse(xml).map_err(|parse_error| {
        error(
            "SEMANTIC_DIFF_XML_INVALID",
            format!("Cannot parse ALS XML for semantic diff: {parse_error}"),
            None,
            None,
        )
    })?;
    if !document.root_element().has_tag_name("Ableton") {
        return Err(error(
            "SEMANTIC_DIFF_ROOT_INVALID",
            "ALS XML has no Ableton root",
            None,
            None,
        ));
    }
    let sample_refs: Vec<_> = document
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("SampleRef"))
        .collect();
    let mut masks = Vec::with_capacity(operations.len() * 3);
    for operation in operations {
        let file_ref = sample_refs
            .get(operation.als_ref_id)
            .copied()
            .and_then(|node| direct_child(node, "FileRef"))
            .ok_or_else(|| {
                error(
                    "SEMANTIC_DIFF_LOCATOR_MISSING",
                    "Approved active FileRef is missing",
                    Some(&operation.operation_id),
                    None,
                )
            })?;
        let values = if use_new_values {
            [
                ("Path", Some(operation.new_path.as_str())),
                ("RelativePath", Some(operation.new_relative_path.as_str())),
                (
                    "RelativePathType",
                    Some(operation.new_relative_path_type.as_str()),
                ),
            ]
        } else {
            [
                ("Path", operation.old_path.as_deref()),
                ("RelativePath", operation.old_relative_path.as_deref()),
                (
                    "RelativePathType",
                    operation.old_relative_path_type.as_deref(),
                ),
            ]
        };
        for (field, expected) in values {
            let expected = expected.ok_or_else(|| {
                error(
                    "SEMANTIC_DIFF_EXPECTATION_MISSING",
                    format!("Expected {field} value is absent"),
                    Some(&operation.operation_id),
                    None,
                )
            })?;
            let attribute = direct_child(file_ref, field)
                .and_then(|node| node.attribute_node("Value"))
                .ok_or_else(|| {
                    error(
                        "SEMANTIC_DIFF_FIELD_MISSING",
                        format!("{field} Value is absent"),
                        Some(&operation.operation_id),
                        None,
                    )
                })?;
            if attribute.value() != expected {
                return Err(error(
                    "SEMANTIC_DIFF_VALUE_MISMATCH",
                    format!("{field} does not equal the approved value"),
                    Some(&operation.operation_id),
                    None,
                ));
            }
            masks.push(Mask {
                range: attribute.range_value(),
                token: format!("__RESCUE_ALLOWED_{}_{}__", operation.operation_id, field),
            });
        }
    }
    masks.sort_by_key(|mask| mask.range.start);
    if masks
        .windows(2)
        .any(|pair| pair[0].range.end > pair[1].range.start)
    {
        return Err(error(
            "SEMANTIC_DIFF_OVERLAPPING_RANGES",
            "Approved semantic diff ranges overlap",
            None,
            None,
        ));
    }
    Ok(masks)
}

fn apply_masks(xml: &str, masks: Vec<Mask>) -> String {
    let mut output = xml.to_string();
    for mask in masks.iter().rev() {
        output.replace_range(mask.range.clone(), &mask.token);
    }
    output
}

fn decode(path: &Path) -> Result<DecodedAls, PackageValidationError> {
    let metadata = fs::symlink_metadata(path).map_err(|read_error| {
        error(
            "VALIDATION_ALS_UNAVAILABLE",
            format!("Cannot inspect ALS: {read_error}"),
            None,
            Some(path),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(error(
            "VALIDATION_ALS_NOT_REGULAR",
            "ALS must be a regular non-symlink file",
            None,
            Some(path),
        ));
    }
    if metadata.len() > MAX_COMPRESSED_ALS_BYTES {
        return Err(error(
            "VALIDATION_ALS_TOO_LARGE",
            "ALS exceeds the compressed size limit",
            None,
            Some(path),
        ));
    }
    let compressed = fs::read(path).map_err(|read_error| {
        error(
            "VALIDATION_ALS_READ_FAILED",
            format!("Cannot read ALS: {read_error}"),
            None,
            Some(path),
        )
    })?;
    let mut decoder = GzDecoder::new(compressed.as_slice());
    let mut limited = decoder
        .by_ref()
        .take(MAX_DECOMPRESSED_XML_BYTES.saturating_add(1));
    let mut xml_bytes = Vec::new();
    limited.read_to_end(&mut xml_bytes).map_err(|read_error| {
        error(
            "VALIDATION_ALS_NOT_GZIP",
            format!("Cannot decompress ALS: {read_error}"),
            None,
            Some(path),
        )
    })?;
    if xml_bytes.len() as u64 > MAX_DECOMPRESSED_XML_BYTES {
        return Err(error(
            "VALIDATION_XML_TOO_LARGE",
            "ALS XML exceeds the decompressed size limit",
            None,
            Some(path),
        ));
    }
    let xml = String::from_utf8(xml_bytes).map_err(|utf8_error| {
        error(
            "VALIDATION_XML_INVALID_UTF8",
            format!("ALS XML is not UTF-8: {utf8_error}"),
            None,
            Some(path),
        )
    })?;
    Ok(DecodedAls {
        hash: format!("{:x}", Sha256::digest(&compressed)),
        xml,
    })
}

fn direct_child<'a, 'input>(
    node: roxmltree::Node<'a, 'input>,
    tag: &str,
) -> Option<roxmltree::Node<'a, 'input>> {
    node.children()
        .find(|child| child.is_element() && child.has_tag_name(tag))
}

fn failed(error: PackageValidationError) -> SemanticDiffOutcome {
    SemanticDiffOutcome {
        records: Vec::new(),
        errors: vec![error],
    }
}

fn error(
    code: &str,
    message: impl Into<String>,
    operation_id: Option<&str>,
    path: Option<&Path>,
) -> PackageValidationError {
    crate::package_validator_result::error(code, message, operation_id, path)
}
