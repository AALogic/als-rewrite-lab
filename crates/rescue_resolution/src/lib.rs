mod asset_resolution;
mod asset_resolution_impl;
mod asset_resolution_result;
mod asset_resolution_scoring;

pub use asset_resolution::{
    resolve_assets, AssetResolutionError, AssetResolutionMetadata, AssetResolutionResult,
    AssetResolutionWarning, ResolutionCandidate, ResolutionDecision, ResolutionEvidence,
    ResolutionProposal, ASSET_RESOLUTION_VERSION, RESOLUTION_POLICY_VERSION,
};
