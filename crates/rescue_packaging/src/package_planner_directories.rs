use crate::CreateDirectoryOperation;
use std::collections::BTreeSet;
use std::path::PathBuf;

pub(crate) fn project_directories() -> Vec<CreateDirectoryOperation> {
    [
        (
            "create_project_info",
            "Ableton Project Info",
            "ableton_project_marker",
        ),
        ("create_samples", "Samples", "ableton_samples_root"),
        (
            "create_imported_samples",
            "Samples/Imported",
            "imported_audio_root",
        ),
    ]
    .into_iter()
    .map(|(operation_id, path, purpose)| CreateDirectoryOperation {
        operation_id: operation_id.to_string(),
        target_relative_path: PathBuf::from(path),
        purpose: purpose.to_string(),
        collision_policy: "fail_if_exists".to_string(),
    })
    .collect()
}

pub(crate) fn add_audio_parent_directories<'a>(
    operations: &mut Vec<CreateDirectoryOperation>,
    targets: impl Iterator<Item = &'a PathBuf>,
) {
    let existing: BTreeSet<_> = operations
        .iter()
        .map(|operation| operation.target_relative_path.clone())
        .collect();
    let mut additional = BTreeSet::new();
    for target in targets {
        let mut parent = target.parent();
        while let Some(path) = parent {
            if path.as_os_str().is_empty() {
                break;
            }
            if !existing.contains(path) {
                additional.insert(path.to_path_buf());
            }
            parent = path.parent();
        }
    }
    let mut additional: Vec<_> = additional.into_iter().collect();
    additional.sort_by(|left, right| {
        left.components()
            .count()
            .cmp(&right.components().count())
            .then_with(|| left.cmp(right))
    });
    for (index, path) in additional.into_iter().enumerate() {
        operations.push(CreateDirectoryOperation {
            operation_id: format!("create_audio_parent_{index:06}"),
            target_relative_path: path,
            purpose: "project_local_audio_parent".to_string(),
            collision_policy: "fail_if_exists".to_string(),
        });
    }
}
