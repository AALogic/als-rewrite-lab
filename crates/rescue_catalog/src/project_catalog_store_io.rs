use crate::{project_catalog_store_validation, ProjectCatalogStoreError, StoredProjectCatalog};
#[cfg(unix)]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) struct ReplacementFailure {
    pub code: &'static str,
    pub message: String,
}

pub(crate) fn read_catalog(
    store_path: &Path,
) -> Result<Option<StoredProjectCatalog>, ProjectCatalogStoreError> {
    let metadata = match fs::symlink_metadata(store_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(io_error(
                "CATALOG_STORE_READ_FAILED",
                "Cannot inspect the private catalog state",
                &error,
            ));
        }
    };
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(project_catalog_store_validation::store_error(
            "CATALOG_STORE_TARGET_SYMLINK",
            "The private catalog target must be a regular non-symlink file.",
        ));
    }
    let bytes = fs::read(store_path).map_err(|error| {
        io_error(
            "CATALOG_STORE_READ_FAILED",
            "Cannot read the private catalog state",
            &error,
        )
    })?;
    let catalog: StoredProjectCatalog = serde_json::from_slice(&bytes).map_err(|_| {
        project_catalog_store_validation::store_error(
            "CATALOG_STORE_DESERIALIZE_FAILED",
            "The private catalog state is not valid JSON for this product.",
        )
    })?;
    project_catalog_store_validation::validate_stored_catalog(&catalog)?;
    Ok(Some(catalog))
}

pub(crate) fn write_catalog(
    store_path: &Path,
    catalog: &StoredProjectCatalog,
) -> Result<(), ProjectCatalogStoreError> {
    prepare_parent(store_path)?;
    validate_target(store_path)?;
    let bytes = serialize(catalog)?;
    let temp = temporary_path(store_path);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| {
            io_error(
                "CATALOG_STORE_TEMP_CREATE_FAILED",
                "Cannot create the private catalog temporary file",
                &error,
            )
        })?;
    if let Err(error) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
        remove_owned(&temp);
        return Err(io_error(
            "CATALOG_STORE_TEMP_WRITE_FAILED",
            "Cannot persist the private catalog temporary file",
            &error,
        ));
    }
    drop(file);
    if let Err(error) = validate_temp(&temp, catalog) {
        remove_owned(&temp);
        return Err(error);
    }
    if let Err(failure) = promote(&temp, store_path) {
        remove_owned(&temp);
        return Err(project_catalog_store_validation::store_error(
            failure.code,
            &failure.message,
        ));
    }
    sync_parent(store_path)?;
    let round_trip = read_catalog(store_path)?;
    if round_trip.as_ref() != Some(catalog) {
        return Err(project_catalog_store_validation::store_error(
            "CATALOG_STORE_ROUND_TRIP_FAILED",
            "The private catalog did not round-trip exactly after persistence.",
        ));
    }
    Ok(())
}

fn prepare_parent(store_path: &Path) -> Result<(), ProjectCatalogStoreError> {
    let parent = store_path.parent().ok_or_else(|| {
        project_catalog_store_validation::store_error(
            "CATALOG_STORE_PARENT_INVALID",
            "The private catalog store has no parent directory.",
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        io_error(
            "CATALOG_STORE_PARENT_INVALID",
            "Cannot create the private catalog parent directory",
            &error,
        )
    })?;
    let metadata = fs::symlink_metadata(parent).map_err(|error| {
        io_error(
            "CATALOG_STORE_PARENT_INVALID",
            "Cannot inspect the private catalog parent directory",
            &error,
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(project_catalog_store_validation::store_error(
            "CATALOG_STORE_PARENT_INVALID",
            "The private catalog parent must be a non-symlink directory.",
        ));
    }
    Ok(())
}

fn validate_target(store_path: &Path) -> Result<(), ProjectCatalogStoreError> {
    match fs::symlink_metadata(store_path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.file_type().is_file() => {
            Err(project_catalog_store_validation::store_error(
                "CATALOG_STORE_TARGET_SYMLINK",
                "The private catalog target must be a regular non-symlink file.",
            ))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error(
            "CATALOG_STORE_PATH_INVALID",
            "Cannot inspect the private catalog target",
            &error,
        )),
    }
}

fn serialize(catalog: &StoredProjectCatalog) -> Result<Vec<u8>, ProjectCatalogStoreError> {
    let mut bytes = serde_json::to_vec_pretty(catalog).map_err(|_| {
        project_catalog_store_validation::store_error(
            "CATALOG_STORE_SERIALIZE_FAILED",
            "The private catalog state could not be serialized.",
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn validate_temp(
    temp: &Path,
    expected: &StoredProjectCatalog,
) -> Result<(), ProjectCatalogStoreError> {
    let bytes = fs::read(temp).map_err(|error| {
        io_error(
            "CATALOG_STORE_TEMP_WRITE_FAILED",
            "Cannot verify the private catalog temporary file",
            &error,
        )
    })?;
    let actual: StoredProjectCatalog = serde_json::from_slice(&bytes).map_err(|_| {
        project_catalog_store_validation::store_error(
            "CATALOG_STORE_TEMP_WRITE_FAILED",
            "The private catalog temporary file is not valid JSON.",
        )
    })?;
    project_catalog_store_validation::validate_stored_catalog(&actual)?;
    if &actual != expected {
        return Err(project_catalog_store_validation::store_error(
            "CATALOG_STORE_TEMP_WRITE_FAILED",
            "The private catalog temporary file differs from the requested state.",
        ));
    }
    Ok(())
}

fn promote(temp: &Path, target: &Path) -> Result<(), ReplacementFailure> {
    if !target.exists() {
        return fs::rename(temp, target).map_err(|error| ReplacementFailure {
            code: "CATALOG_STORE_ATOMIC_REPLACE_FAILED",
            message: io_message("Cannot create the private catalog state", &error),
        });
    }
    promote_existing(temp, target)
}

#[cfg(unix)]
fn promote_existing(temp: &Path, target: &Path) -> Result<(), ReplacementFailure> {
    fs::rename(temp, target).map_err(|error| ReplacementFailure {
        code: "CATALOG_STORE_ATOMIC_REPLACE_FAILED",
        message: io_message(
            "Cannot atomically replace the private catalog state",
            &error,
        ),
    })
}

#[cfg(windows)]
fn promote_existing(temp: &Path, target: &Path) -> Result<(), ReplacementFailure> {
    crate::project_catalog_store_windows::replace_file(temp, target)
}

#[cfg(not(any(unix, windows)))]
fn promote_existing(_temp: &Path, _target: &Path) -> Result<(), ReplacementFailure> {
    Err(ReplacementFailure {
        code: "CATALOG_STORE_ATOMIC_REPLACE_FAILED",
        message: "Atomic private catalog replacement is unsupported on this platform.".to_string(),
    })
}

#[cfg(unix)]
fn sync_parent(store_path: &Path) -> Result<(), ProjectCatalogStoreError> {
    let parent = store_path.parent().ok_or_else(|| {
        project_catalog_store_validation::store_error(
            "CATALOG_STORE_PARENT_INVALID",
            "The private catalog store has no parent directory.",
        )
    })?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            io_error(
                "CATALOG_STORE_ATOMIC_REPLACE_FAILED",
                "Cannot sync the private catalog parent directory",
                &error,
            )
        })
}

#[cfg(not(unix))]
fn sync_parent(_store_path: &Path) -> Result<(), ProjectCatalogStoreError> {
    Ok(())
}

fn temporary_path(target: &Path) -> PathBuf {
    target.with_file_name(".ableton-project-rescue.catalog.tmp")
}

fn remove_owned(path: &Path) {
    let _ = fs::remove_file(path);
}

fn io_error(code: &str, message: &str, error: &std::io::Error) -> ProjectCatalogStoreError {
    project_catalog_store_validation::store_error(code, &io_message(message, error))
}

fn io_message(message: &str, error: &std::io::Error) -> String {
    match error.raw_os_error() {
        Some(code) => format!("{message}; io_kind={:?}; os_error={code}", error.kind()),
        None => format!("{message}; io_kind={:?}", error.kind()),
    }
}
