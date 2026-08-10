use rescue_core::DependencyRef;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ReferenceClaim {
    dependency_kind: String,
    source_kind: String,
    raw_path: Option<String>,
    raw_relative_path: Option<String>,
    relative_path_type: Option<String>,
    file_type: Option<String>,
    filename: Option<String>,
    extension: Option<String>,
    original_file_size: Option<String>,
    original_crc: Option<String>,
    incomplete_occurrence: Option<String>,
}

pub(crate) struct OccurrenceGroup {
    pub basis: &'static str,
    pub dependency_indexes: Vec<usize>,
}

pub(crate) fn group_occurrences(dependencies: &[DependencyRef]) -> Vec<OccurrenceGroup> {
    let mut index_by_claim: BTreeMap<ReferenceClaim, usize> = BTreeMap::new();
    let mut groups: Vec<OccurrenceGroup> = Vec::new();

    for (dependency_index, dependency) in dependencies.iter().enumerate() {
        let complete = has_complete_grouping_claim(dependency);
        let claim = ReferenceClaim {
            dependency_kind: dependency.dependency_kind.clone(),
            source_kind: dependency.source_kind.clone(),
            raw_path: dependency.raw_path.clone(),
            raw_relative_path: dependency.raw_relative_path.clone(),
            relative_path_type: dependency.relative_path_type.clone(),
            file_type: dependency.file_type.clone(),
            filename: dependency.filename.clone(),
            extension: dependency.extension.clone(),
            original_file_size: dependency.original_file_size.clone(),
            original_crc: dependency.original_crc.clone(),
            incomplete_occurrence: (!complete).then(|| dependency.dependency_id.clone()),
        };
        if let Some(group_index) = index_by_claim.get(&claim).copied() {
            groups[group_index]
                .dependency_indexes
                .push(dependency_index);
        } else {
            let group_index = groups.len();
            index_by_claim.insert(claim, group_index);
            groups.push(OccurrenceGroup {
                basis: if complete {
                    "exact_recorded_reference_claim"
                } else {
                    "single_incomplete_occurrence"
                },
                dependency_indexes: vec![dependency_index],
            });
        }
    }
    groups
}

fn has_complete_grouping_claim(dependency: &DependencyRef) -> bool {
    let has_path = dependency
        .raw_path
        .as_deref()
        .is_some_and(|value| !value.is_empty())
        || dependency
            .raw_relative_path
            .as_deref()
            .is_some_and(|value| !value.is_empty());
    let has_filename = dependency
        .filename
        .as_deref()
        .is_some_and(|value| !value.is_empty());
    has_path && has_filename
}
