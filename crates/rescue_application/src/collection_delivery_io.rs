use crate::{delivery_error, CollectionDeliveryError, CollectionDeliveryOperation};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TreeFile {
    relative_path: PathBuf,
    byte_size: u64,
}

pub(crate) fn copy_and_promote(
    operation: &CollectionDeliveryOperation,
) -> Result<(usize, u64), CollectionDeliveryError> {
    if operation.target_root.exists() || operation.staging_root.exists() {
        return Err(delivery_error(
            "DELIVERY_STAGING_EXISTS",
            "execute",
            "The delivery target or staging directory is no longer available.",
        ));
    }
    let source_tree = read_tree(&operation.source_root)?;
    std::fs::create_dir(&operation.staging_root).map_err(|_| copy_error())?;
    for file in source_tree.values() {
        let source = operation.source_root.join(&file.relative_path);
        let target = operation.staging_root.join(&file.relative_path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|_| copy_error())?;
        }
        std::fs::copy(source, target).map_err(|_| copy_error())?;
    }
    let staged_tree = read_tree(&operation.staging_root)?;
    if source_tree != staged_tree {
        return Err(delivery_error(
            "DELIVERY_VALIDATION_FAILED",
            "validate",
            "The staged Project does not match the source tree.",
        ));
    }
    if operation.target_root.exists() {
        return Err(delivery_error(
            "DELIVERY_PROMOTION_FAILED",
            "promote",
            "The delivery target became occupied before promotion.",
        ));
    }
    std::fs::rename(&operation.staging_root, &operation.target_root).map_err(|_| {
        delivery_error(
            "DELIVERY_PROMOTION_FAILED",
            "promote",
            "The validated Project could not be promoted.",
        )
    })?;
    Ok((
        source_tree.len(),
        source_tree.values().map(|file| file.byte_size).sum(),
    ))
}

pub(crate) fn read_tree(
    root: &Path,
) -> Result<BTreeMap<PathBuf, TreeFile>, CollectionDeliveryError> {
    let mut files = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let entries = std::fs::read_dir(&directory).map_err(|_| source_error())?;
        for entry in entries {
            let entry = entry.map_err(|_| source_error())?;
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path).map_err(|_| source_error())?;
            if metadata.file_type().is_symlink() {
                return Err(delivery_error(
                    "DELIVERY_SOURCE_CONTAINS_LINK",
                    "inspect",
                    "The source Project contains a symbolic link.",
                ));
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|_| source_error())?
                    .to_path_buf();
                files.insert(
                    relative.clone(),
                    TreeFile {
                        relative_path: relative,
                        byte_size: metadata.len(),
                    },
                );
            } else {
                return Err(source_error());
            }
        }
    }
    Ok(files)
}

pub(crate) fn validate_real_directory(
    path: &Path,
    code: &str,
    stage: &str,
) -> Result<(), CollectionDeliveryError> {
    if !path.is_absolute() {
        return Err(delivery_error(
            code,
            stage,
            "A required local directory is invalid.",
        ));
    }
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| delivery_error(code, stage, "A required local directory is unavailable."))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(delivery_error(
            code,
            stage,
            "A required local directory is invalid.",
        ));
    }
    Ok(())
}

pub(crate) fn cleanup_owned_staging(path: &Path) {
    let is_owned = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.starts_with(".als-rescue-delivery-") && name.ends_with(".staging")
        });
    if is_owned {
        let _ = std::fs::remove_dir_all(path);
    }
}

fn source_error() -> CollectionDeliveryError {
    delivery_error(
        "DELIVERY_SOURCE_INVALID",
        "inspect",
        "The source Project tree is unsupported.",
    )
}

fn copy_error() -> CollectionDeliveryError {
    delivery_error(
        "DELIVERY_COPY_FAILED",
        "copy",
        "The Project could not be copied to staging.",
    )
}
