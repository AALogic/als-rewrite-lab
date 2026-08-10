mod asset_file_snapshot;
mod asset_inventory;
mod asset_inventory_hash;
mod asset_inventory_impl;
mod asset_inventory_walk;
mod project_catalog;
mod project_catalog_grouping;
mod project_catalog_identity;
mod project_catalog_impl;
mod project_catalog_store;
mod project_catalog_store_impl;
mod project_catalog_store_io;
mod project_catalog_store_merge;
mod project_catalog_store_validation;
#[cfg(windows)]
mod project_catalog_store_windows;
mod project_catalog_validation;
mod project_scan;
mod project_scan_control;
mod project_scan_impl;
mod project_scan_observe;
mod project_scan_walk;

pub use asset_file_snapshot::{snapshot_asset_files, AssetFileSnapshotRequest};
pub use asset_inventory::{
    scan_assets, AssetInventoryError, AssetInventoryMetadata, AssetInventoryRequest,
    AssetInventoryResult, AssetInventoryWarning, ContentRecord, FileOccurrence,
    ASSET_INVENTORY_VERSION,
};
pub use project_catalog::{
    build_project_catalog, LiveSetRecord, ProjectCatalogBuildRequest, ProjectCatalogError,
    ProjectCatalogMetadata, ProjectCatalogSnapshot, ProjectCatalogWarning, ProjectFolderRecord,
    PROJECT_CATALOG_IDENTITY_POLICY, PROJECT_CATALOG_VERSION,
};
pub use project_catalog_store::{
    load_project_catalog, store_project_catalog, ProjectCatalogLoadResult,
    ProjectCatalogStoreError, ProjectCatalogStoreMetadata, ProjectCatalogStoreRequest,
    ProjectCatalogStoreResult, ProjectCatalogStoreWarning, StoredLiveSetRecord,
    StoredProjectCatalog, StoredProjectFolderRecord, PROJECT_CATALOG_STORAGE_SCHEMA_VERSION,
};
pub use project_scan::{
    scan_projects, scan_projects_controlled, ALSFileObservation, ProjectMarkerObservation,
    ProjectScanError, ProjectScanMetadata, ProjectScanProgress, ProjectScanRequest,
    ProjectScanResult, ProjectScanWarning, PROJECT_SCANNER_VERSION, PROJECT_SCAN_TRAVERSAL_POLICY,
};
pub use project_scan_control::ProjectScanObserver;
