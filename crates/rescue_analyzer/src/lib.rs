mod project_discovery;
mod project_discovery_impl;

pub use project_discovery::{
    discover_project, ProjectDiscoveryError, ProjectDiscoveryMetadata, ProjectDiscoveryRequest,
    ProjectDiscoveryResult, ProjectDiscoveryWarning, ProjectRootCandidate,
    PROJECT_DISCOVERY_VERSION,
};
