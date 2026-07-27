use rescue_packaging::RewriteOperation;
use std::collections::BTreeSet;
use std::ops::Range;

pub(crate) struct XmlRewriteOutcome {
    pub xml: String,
    pub completed_operation_ids: Vec<String>,
}

pub(crate) struct XmlRewriteFailure {
    pub code: &'static str,
    pub message: String,
    pub operation_id: Option<String>,
}

struct Replacement {
    range: Range<usize>,
    encoded_value: String,
}

pub(crate) fn rewrite_xml(
    xml: &str,
    operations: &[RewriteOperation],
) -> Result<XmlRewriteOutcome, XmlRewriteFailure> {
    let document = roxmltree::Document::parse(xml).map_err(|error| {
        failure(
            "REWRITE_INPUT_XML_INVALID",
            format!("Cannot parse staged ALS XML: {error}"),
            None,
        )
    })?;
    if !document.root_element().has_tag_name("Ableton") {
        return Err(failure(
            "REWRITE_INPUT_ROOT_INVALID",
            "Staged ALS XML has no Ableton root".to_string(),
            None,
        ));
    }
    validate_operation_identity(operations)?;
    let sample_refs: Vec<_> = document
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("SampleRef"))
        .collect();
    let mut replacements = Vec::with_capacity(operations.len() * 3);
    for operation in operations {
        let sample_ref = sample_refs
            .get(operation.als_ref_id)
            .copied()
            .ok_or_else(|| {
                failure(
                    "REWRITE_LOCATOR_NOT_FOUND",
                    "Planned SampleRef index does not exist".to_string(),
                    Some(operation.operation_id.clone()),
                )
            })?;
        let expected_locator = format!("SampleRef[{}]/FileRef", operation.als_ref_id);
        if operation.xml_locator != expected_locator {
            return Err(failure(
                "REWRITE_LOCATOR_MISMATCH",
                "Rewrite locator does not match the snapshot-bound reference ID".to_string(),
                Some(operation.operation_id.clone()),
            ));
        }
        let file_ref = direct_child(sample_ref, "FileRef").ok_or_else(|| {
            failure(
                "REWRITE_FILEREF_NOT_FOUND",
                "SampleRef has no direct FileRef child".to_string(),
                Some(operation.operation_id.clone()),
            )
        })?;
        collect_replacement(
            file_ref,
            "Path",
            operation.old_path.as_deref(),
            &operation.new_path,
            operation,
            &mut replacements,
        )?;
        collect_replacement(
            file_ref,
            "RelativePath",
            operation.old_relative_path.as_deref(),
            &operation.new_relative_path,
            operation,
            &mut replacements,
        )?;
        collect_replacement(
            file_ref,
            "RelativePathType",
            operation.old_relative_path_type.as_deref(),
            &operation.new_relative_path_type,
            operation,
            &mut replacements,
        )?;
    }
    ensure_non_overlapping(&mut replacements)?;
    let mut rewritten = xml.to_string();
    for replacement in replacements.iter().rev() {
        rewritten.replace_range(replacement.range.clone(), &replacement.encoded_value);
    }
    verify_rewritten_values(&rewritten, operations)?;
    Ok(XmlRewriteOutcome {
        xml: rewritten,
        completed_operation_ids: operations
            .iter()
            .map(|operation| operation.operation_id.clone())
            .collect(),
    })
}

fn validate_operation_identity(operations: &[RewriteOperation]) -> Result<(), XmlRewriteFailure> {
    let mut ids = BTreeSet::new();
    let mut refs = BTreeSet::new();
    for operation in operations {
        if !ids.insert(operation.operation_id.as_str()) {
            return Err(failure(
                "REWRITE_DUPLICATE_OPERATION_ID",
                "Rewrite operation IDs must be unique".to_string(),
                Some(operation.operation_id.clone()),
            ));
        }
        if !refs.insert(operation.als_ref_id) {
            return Err(failure(
                "REWRITE_DUPLICATE_REFERENCE",
                "A single active reference may be rewritten only once".to_string(),
                Some(operation.operation_id.clone()),
            ));
        }
    }
    Ok(())
}

fn collect_replacement(
    file_ref: roxmltree::Node<'_, '_>,
    field: &str,
    expected_old: Option<&str>,
    new_value: &str,
    operation: &RewriteOperation,
    replacements: &mut Vec<Replacement>,
) -> Result<(), XmlRewriteFailure> {
    let expected_old = expected_old.ok_or_else(|| {
        failure(
            "REWRITE_OLD_VALUE_MISSING",
            format!("Plan has no old value for {field}"),
            Some(operation.operation_id.clone()),
        )
    })?;
    let value_node = direct_child(file_ref, field).ok_or_else(|| {
        failure(
            "REWRITE_FIELD_NOT_FOUND",
            format!("FileRef has no direct {field} child"),
            Some(operation.operation_id.clone()),
        )
    })?;
    let attribute = value_node.attribute_node("Value").ok_or_else(|| {
        failure(
            "REWRITE_VALUE_ATTRIBUTE_NOT_FOUND",
            format!("{field} has no Value attribute"),
            Some(operation.operation_id.clone()),
        )
    })?;
    if attribute.value() != expected_old {
        return Err(failure(
            "REWRITE_OLD_VALUE_MISMATCH",
            format!("Current {field} value does not match the approved plan"),
            Some(operation.operation_id.clone()),
        ));
    }
    replacements.push(Replacement {
        range: attribute.range_value(),
        encoded_value: escape_xml_attribute(new_value),
    });
    Ok(())
}

fn ensure_non_overlapping(replacements: &mut [Replacement]) -> Result<(), XmlRewriteFailure> {
    replacements.sort_by_key(|replacement| replacement.range.start);
    for pair in replacements.windows(2) {
        if pair[0].range.end > pair[1].range.start {
            return Err(failure(
                "REWRITE_OVERLAPPING_RANGES",
                "Rewrite operations address overlapping XML byte ranges".to_string(),
                None,
            ));
        }
    }
    Ok(())
}

fn verify_rewritten_values(
    xml: &str,
    operations: &[RewriteOperation],
) -> Result<(), XmlRewriteFailure> {
    let document = roxmltree::Document::parse(xml).map_err(|error| {
        failure(
            "REWRITE_OUTPUT_XML_INVALID",
            format!("Rewritten XML does not parse: {error}"),
            None,
        )
    })?;
    let sample_refs: Vec<_> = document
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("SampleRef"))
        .collect();
    for operation in operations {
        let file_ref = sample_refs
            .get(operation.als_ref_id)
            .copied()
            .and_then(|sample_ref| direct_child(sample_ref, "FileRef"))
            .ok_or_else(|| {
                failure(
                    "REWRITE_OUTPUT_LOCATOR_MISSING",
                    "Rewritten output lost a planned FileRef".to_string(),
                    Some(operation.operation_id.clone()),
                )
            })?;
        for (field, expected) in [
            ("Path", operation.new_path.as_str()),
            ("RelativePath", operation.new_relative_path.as_str()),
            (
                "RelativePathType",
                operation.new_relative_path_type.as_str(),
            ),
        ] {
            let observed = direct_child(file_ref, field)
                .and_then(|node| node.attribute("Value"))
                .unwrap_or("");
            if observed != expected {
                return Err(failure(
                    "REWRITE_OUTPUT_VALUE_MISMATCH",
                    format!("Rewritten {field} does not match the approved plan"),
                    Some(operation.operation_id.clone()),
                ));
            }
        }
    }
    Ok(())
}

fn direct_child<'a, 'input>(
    node: roxmltree::Node<'a, 'input>,
    tag_name: &str,
) -> Option<roxmltree::Node<'a, 'input>> {
    node.children()
        .find(|child| child.is_element() && child.has_tag_name(tag_name))
}

fn escape_xml_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn failure(code: &'static str, message: String, operation_id: Option<String>) -> XmlRewriteFailure {
    XmlRewriteFailure {
        code,
        message,
        operation_id,
    }
}
