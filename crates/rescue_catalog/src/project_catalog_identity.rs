use crate::ProjectCatalogError;
use sha2::{Digest, Sha256};
use std::path::Path;

pub(crate) fn project_folder_id(path: &Path) -> Result<String, ProjectCatalogError> {
    hash_path("project-folder", path)
}

pub(crate) fn live_set_id(path: &Path) -> Result<String, ProjectCatalogError> {
    hash_path("live-set", path)
}

pub(crate) fn observation_fingerprint(
    path: &Path,
    file_size: u64,
    modified_time_unix_ms: Option<u64>,
) -> Result<String, ProjectCatalogError> {
    let path = exact_path(path)?;
    let modified = modified_time_unix_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string());
    Ok(hash_fields(&[
        "live-set-observation",
        path,
        &file_size.to_string(),
        &modified,
    ]))
}

pub(crate) fn exact_path(path: &Path) -> Result<&str, ProjectCatalogError> {
    path.to_str().ok_or_else(|| ProjectCatalogError {
        error_code: "CATALOG_PATH_NOT_UTF8".to_string(),
        message: "A catalog input path cannot be represented exactly as UTF-8.".to_string(),
    })
}

fn hash_path(domain: &str, path: &Path) -> Result<String, ProjectCatalogError> {
    Ok(hash_fields(&[domain, exact_path(path)?]))
}

fn hash_fields(fields: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for field in fields {
        hasher.update((field.len() as u64).to_le_bytes());
        hasher.update(field.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}
