use rescue_application::{
    execute_collection_delivery, plan_collection_delivery, CollectionDeliveryRequest,
    CourierCollectionItem, DeliveryCollectionSnapshot,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn project(root: &Path, name: &str) -> PathBuf {
    let project = root.join(name);
    std::fs::create_dir_all(project.join("Samples/Imported")).expect("project dirs");
    std::fs::write(project.join(format!("{name}.als")), b"als").expect("als");
    std::fs::write(project.join("Samples/Imported/kick.wav"), b"audio").expect("audio");
    project
}

fn snapshot(sources: &[PathBuf]) -> DeliveryCollectionSnapshot {
    DeliveryCollectionSnapshot {
        schema_version: "0.1".to_string(),
        collection_id: "collection-1".to_string(),
        revision: 1,
        items: sources
            .iter()
            .enumerate()
            .map(|(index, source)| CourierCollectionItem {
                work_item_id: format!("item-{index}"),
                source_display_name: format!("Set-{index}.als"),
                copy_result_request_id: format!("copy-{index}"),
                target_project_root: source.clone(),
                outcome: "completed".to_string(),
                omitted_asset_count: 0,
            })
            .collect(),
    }
}

fn request(sources: &[PathBuf], destination: &Path, consent: bool) -> CollectionDeliveryRequest {
    CollectionDeliveryRequest {
        request_id: "delivery-1".to_string(),
        snapshot: snapshot(sources),
        destination_root: destination.to_path_buf(),
        previously_delivered_item_ids: Vec::new(),
        write_consent: consent,
    }
}

fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut result = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("tree directory") {
            let path = entry.expect("tree entry").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                result.insert(
                    path.strip_prefix(root).expect("relative").to_path_buf(),
                    std::fs::read(path).expect("file"),
                );
            }
        }
    }
    result
}

#[test]
fn delivery_plan_is_read_only_and_required() {
    let temp = tempfile::tempdir().expect("fixture");
    let source = project(temp.path(), "Source Project");
    let destination = temp.path().join("Delivery");
    std::fs::create_dir(&destination).expect("destination");
    let before = tree(temp.path());
    let request = request(&[source], &destination, true);
    let plan = plan_collection_delivery(&request);
    assert_eq!(tree(temp.path()), before);
    let mut changed = plan.clone();
    changed.collection_revision = 99;
    let result = execute_collection_delivery(&request, &changed);
    assert_eq!(result.error_codes, ["DELIVERY_PLAN_INVALID"]);
}

#[test]
fn delivery_requires_explicit_write_consent() {
    let temp = tempfile::tempdir().expect("fixture");
    let source = project(temp.path(), "Source Project");
    let destination = temp.path().join("Delivery");
    std::fs::create_dir(&destination).expect("destination");
    let request = request(&[source], &destination, false);
    let result = execute_collection_delivery(&request, &plan_collection_delivery(&request));
    assert_eq!(result.error_codes, ["DELIVERY_CONSENT_REQUIRED"]);
    assert!(std::fs::read_dir(destination)
        .expect("empty")
        .next()
        .is_none());
}

#[test]
fn delivery_never_overwrites_existing_target() {
    let temp = tempfile::tempdir().expect("fixture");
    let source = project(temp.path(), "Source Project");
    let destination = temp.path().join("Delivery");
    std::fs::create_dir(&destination).expect("destination");
    let request = request(&[source], &destination, true);
    let plan = plan_collection_delivery(&request);
    std::fs::create_dir_all(&plan.operations[0].target_root).expect("race collision");
    std::fs::write(plan.operations[0].target_root.join("keep.txt"), b"keep").expect("sentinel");
    let result = execute_collection_delivery(&request, &plan);
    assert_eq!(result.items[0].status, "failed");
    assert_eq!(
        std::fs::read(plan.operations[0].target_root.join("keep.txt")).ok(),
        Some(b"keep".to_vec())
    );
}

#[test]
fn collision_suffix_is_deterministic_and_recorded() {
    let temp = tempfile::tempdir().expect("fixture");
    let source = project(temp.path(), "Source Project");
    let destination = temp.path().join("Delivery");
    std::fs::create_dir_all(destination.join("Source Project")).expect("collision");
    let request = request(&[source], &destination, true);
    let plan = plan_collection_delivery(&request);
    assert_eq!(
        plan.operations[0]
            .target_root
            .file_name()
            .and_then(|name| name.to_str()),
        Some("Source Project (2)")
    );
}

#[test]
fn validated_delivery_preserves_project_tree() {
    let temp = tempfile::tempdir().expect("fixture");
    let source = project(temp.path(), "Source Project");
    let destination = temp.path().join("Delivery");
    std::fs::create_dir(&destination).expect("destination");
    let request = request(std::slice::from_ref(&source), &destination, true);
    let plan = plan_collection_delivery(&request);
    let result = execute_collection_delivery(&request, &plan);
    assert_eq!(result.run_status, "completed");
    let target = result.items[0].final_target_root.as_ref().expect("target");
    assert_eq!(tree(&source), tree(target));
    assert_eq!(result.items[0].file_count, 2);
}

#[test]
fn delivery_never_mutates_source_collection() {
    let temp = tempfile::tempdir().expect("fixture");
    let source = project(temp.path(), "Source Project");
    let destination = temp.path().join("Delivery");
    std::fs::create_dir(&destination).expect("destination");
    let before = tree(&source);
    let request = request(std::slice::from_ref(&source), &destination, true);
    let plan = plan_collection_delivery(&request);
    let _ = execute_collection_delivery(&request, &plan);
    assert_eq!(tree(&source), before);
}

#[test]
fn failed_item_does_not_remove_completed_delivery() {
    let temp = tempfile::tempdir().expect("fixture");
    let first = project(temp.path(), "First Project");
    let second = project(temp.path(), "Second Project");
    let destination = temp.path().join("Delivery");
    std::fs::create_dir(&destination).expect("destination");
    let request = request(&[first, second], &destination, true);
    let plan = plan_collection_delivery(&request);
    std::fs::create_dir(&plan.operations[1].target_root).expect("second collision");
    let result = execute_collection_delivery(&request, &plan);
    assert_eq!(result.run_status, "completed_with_issues");
    assert_eq!(result.items[0].status, "completed");
    assert!(result.items[0]
        .final_target_root
        .as_ref()
        .is_some_and(|path| path.is_dir()));
    assert_eq!(result.items[1].status, "failed");
}

#[test]
fn previously_delivered_items_are_skipped() {
    let temp = tempfile::tempdir().expect("fixture");
    let first = project(temp.path(), "First Project");
    let second = project(temp.path(), "Second Project");
    let destination = temp.path().join("Delivery");
    std::fs::create_dir(&destination).expect("destination");
    let mut request = request(&[first, second], &destination, true);
    request.previously_delivered_item_ids = vec!["item-0".to_string()];
    let plan = plan_collection_delivery(&request);
    assert_eq!(plan.skipped_item_ids, ["item-0"]);
    assert_eq!(plan.operations.len(), 1);
}

#[cfg(unix)]
#[test]
fn symlink_and_special_file_sources_fail_closed() {
    let temp = tempfile::tempdir().expect("fixture");
    let source = project(temp.path(), "Source Project");
    std::os::unix::fs::symlink(source.join("Source Project.als"), source.join("link.als"))
        .expect("source link");
    let destination = temp.path().join("Delivery");
    std::fs::create_dir(&destination).expect("destination");
    let request = request(&[source], &destination, true);
    let plan = plan_collection_delivery(&request);
    assert!(plan.operations.is_empty());
    assert_eq!(plan.errors[0].error_code, "DELIVERY_SOURCE_CONTAINS_LINK");
}

#[test]
fn delivery_diagnostics_are_path_free() {
    let secret = PathBuf::from("/Users/private/Secret Delivery");
    let request = CollectionDeliveryRequest {
        request_id: "delivery".to_string(),
        snapshot: snapshot(&[]),
        destination_root: secret.clone(),
        previously_delivered_item_ids: Vec::new(),
        write_consent: false,
    };
    let plan = plan_collection_delivery(&request);
    let json = serde_json::to_string(&plan.errors).expect("errors");
    assert!(!json.contains(&secret.to_string_lossy().to_string()));
}
