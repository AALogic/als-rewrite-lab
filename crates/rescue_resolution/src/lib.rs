mod asset_resolution;
mod asset_resolution_impl;
mod asset_resolution_result;
mod asset_resolution_scoring;
mod asset_resolution_selection;
mod current_path_binding;

pub use asset_resolution::{
    resolve_assets, resolve_assets_with_selections, AssetResolutionError, AssetResolutionMetadata,
    AssetResolutionResult, AssetResolutionWarning, ResolutionCandidate, ResolutionDecision,
    ResolutionEvidence, ResolutionProposal, UserAssetSelection, UserSelectionSet,
    ASSET_RESOLUTION_VERSION, RESOLUTION_POLICY_VERSION, USER_SELECTION_SCHEMA_VERSION,
};
pub use current_path_binding::{
    bind_current_paths, CurrentPathBinding, CurrentPathBindingError, CurrentPathBindingMetadata,
    CurrentPathBindingOmission, CurrentPathBindingResult, CURRENT_PATH_BINDING_POLICY_VERSION,
    CURRENT_PATH_BINDING_VERSION,
};
