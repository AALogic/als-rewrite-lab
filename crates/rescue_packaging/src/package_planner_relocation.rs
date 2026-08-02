use rescue_core::ActiveAudioReference;
use std::path::{Component, Path, PathBuf};

pub(crate) struct ReferenceRelocation {
    pub target_relative: PathBuf,
    pub new_relative_path: String,
    pub new_relative_path_type: String,
    pub fields_to_change: Vec<String>,
    pub support_status: &'static str,
}

pub(crate) fn imported_target(filename: &str) -> Option<PathBuf> {
    let path = Path::new(filename);
    if path.is_absolute()
        || path.components().count() != 1
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    Some(PathBuf::from("Samples").join("Imported").join(path))
}

pub(crate) fn relocation_for(
    reference: &ActiveAudioReference,
    filename: &str,
) -> Result<ReferenceRelocation, &'static str> {
    if !supported_reference_base(reference) {
        return Err("rewrite_reference_not_supported");
    }
    match reference.relative_path_type.as_deref() {
        Some("1") => external_relocation(filename),
        Some("3") => project_local_relocation(reference),
        _ => Err("rewrite_reference_not_supported"),
    }
}

fn external_relocation(filename: &str) -> Result<ReferenceRelocation, &'static str> {
    let target = imported_target(filename).ok_or("unsafe_or_empty_target_filename")?;
    let new_relative_path = target
        .to_str()
        .ok_or("target_relative_path_is_not_unicode")?
        .to_string();
    Ok(ReferenceRelocation {
        target_relative: target,
        new_relative_path,
        new_relative_path_type: "3".to_string(),
        fields_to_change: vec![
            "Path".to_string(),
            "RelativePath".to_string(),
            "RelativePathType".to_string(),
        ],
        support_status: "experimental_lab_only",
    })
}

fn project_local_relocation(
    reference: &ActiveAudioReference,
) -> Result<ReferenceRelocation, &'static str> {
    let raw_relative = reference
        .raw_relative_path
        .as_deref()
        .ok_or("project_relative_path_missing")?;
    let target =
        project_local_target(raw_relative).ok_or("project_relative_path_unsafe_or_unsupported")?;
    Ok(ReferenceRelocation {
        target_relative: target,
        new_relative_path: raw_relative.to_string(),
        new_relative_path_type: "3".to_string(),
        fields_to_change: vec!["Path".to_string()],
        support_status: "confirmed_lab",
    })
}

fn project_local_target(raw_relative_path: &str) -> Option<PathBuf> {
    let normalized = raw_relative_path.replace('\\', "/");
    let components: Vec<_> = normalized.split('/').collect();
    if components.len() < 2
        || components.first() != Some(&"Samples")
        || components
            .iter()
            .any(|component| component.is_empty() || matches!(*component, "." | ".."))
    {
        return None;
    }
    let mut target = PathBuf::new();
    for component in components {
        target.push(component);
    }
    Some(target)
}

fn supported_reference_base(reference: &ActiveAudioReference) -> bool {
    reference.is_rewrite_candidate
        && reference.rewrite_support_status == "supported"
        && matches!(
            reference.usage_context.as_str(),
            "audio_clip" | "simpler_multisample"
        )
        && reference.xml_locator == format!("SampleRef[{}]/FileRef", reference.ref_id)
        && reference.xml_locator.starts_with("SampleRef[")
        && reference.xml_locator.ends_with("]/FileRef")
}
