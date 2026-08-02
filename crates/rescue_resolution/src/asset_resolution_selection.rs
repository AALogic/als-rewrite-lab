use crate::{
    ResolutionCandidate, UserAssetSelection, UserSelectionSet, USER_SELECTION_SCHEMA_VERSION,
};
use rescue_analyzer::DependencyAssessmentResult;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Component;

pub(crate) struct ValidatedSelections<'a> {
    by_asset: BTreeMap<&'a str, &'a UserAssetSelection>,
}

impl<'a> ValidatedSelections<'a> {
    pub(crate) fn empty() -> Self {
        Self {
            by_asset: BTreeMap::new(),
        }
    }

    pub(crate) fn get(&self, required_asset_id: &str) -> Option<&'a UserAssetSelection> {
        self.by_asset.get(required_asset_id).copied()
    }
}

pub(crate) fn validate<'a>(
    assessment: &DependencyAssessmentResult,
    selections: Option<&'a UserSelectionSet>,
) -> Result<ValidatedSelections<'a>, (&'static str, &'static str)> {
    let Some(selections) = selections else {
        return Ok(ValidatedSelections::empty());
    };
    if selections.selection_schema_version != USER_SELECTION_SCHEMA_VERSION {
        return Err((
            "RESOLUTION_SELECTION_SCHEMA_UNSUPPORTED",
            "User selection schema version is unsupported",
        ));
    }
    if selections.source_als_sha256 != assessment.assessment_metadata.source_file_hash {
        return Err((
            "RESOLUTION_SELECTION_SOURCE_MISMATCH",
            "User selections describe a different ALS snapshot",
        ));
    }
    let known_assets: BTreeSet<_> = assessment
        .required_assets
        .iter()
        .map(|asset| asset.required_asset_id.as_str())
        .collect();
    let mut by_asset = BTreeMap::new();
    for selection in &selections.selections {
        if !known_assets.contains(selection.required_asset_id.as_str()) {
            return Err((
                "RESOLUTION_SELECTION_UNKNOWN_ASSET",
                "User selection refers to an unknown required asset",
            ));
        }
        if !safe_selection(selection) {
            return Err((
                "RESOLUTION_SELECTION_VALUE_INVALID",
                "User selection path or SHA-256 is invalid",
            ));
        }
        if by_asset
            .insert(selection.required_asset_id.as_str(), selection)
            .is_some()
        {
            return Err((
                "RESOLUTION_SELECTION_DUPLICATE_ASSET",
                "User selection contains duplicate decisions for one asset",
            ));
        }
    }
    Ok(ValidatedSelections { by_asset })
}

pub(crate) fn matching_candidate<'a>(
    candidates: &'a [ResolutionCandidate],
    selection: &UserAssetSelection,
) -> Option<&'a ResolutionCandidate> {
    let expected_content_id = format!("sha256:{}", selection.selected_content_sha256);
    let mut matching = candidates.iter().filter(|candidate| {
        candidate.native_path == selection.selected_native_path
            && candidate.content_id == expected_content_id
            && candidate.conflicts.is_empty()
    });
    let candidate = matching.next()?;
    matching.next().is_none().then_some(candidate)
}

fn safe_selection(selection: &UserAssetSelection) -> bool {
    selection.selected_native_path.is_absolute()
        && !selection
            .selected_native_path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        && selection.selected_content_sha256.len() == 64
        && selection
            .selected_content_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
