use super::*;

fn request(provider_id: &str) -> ExternalProviderOpenRequest {
    ExternalProviderOpenRequest {
        request_id: "provider-open-1".to_string(),
        provider_id: provider_id.to_string(),
    }
}

#[test]
fn unsupported_provider_is_rejected_before_effects() {
    let failure = validate_request(&request("https://example.invalid"))
        .expect_err("provider mapping must remain closed");
    assert_eq!(failure.error_code, "PROVIDER_OPEN_UNSUPPORTED");
}

#[test]
fn trusted_provider_opens_without_arming_payload() {
    let trusted = request(PROVIDER_ID);
    assert!(validate_request(&trusted).is_ok());
    let result = opened(&trusted);
    assert_eq!(result.state, "opened");
    let source = include_str!("external_folder_handoff_commands.rs");
    assert!(!source.contains("TransferPayloadState"));
    assert!(!source.contains("prepare_attempt"));
}

#[test]
fn provider_open_wire_contract_is_path_free() {
    let trusted = request(PROVIDER_ID);
    let request_json = serde_json::to_value(&trusted).expect("serialize request");
    let result_json = serde_json::to_value(opened(&trusted)).expect("serialize result");
    let error_json = serde_json::to_value(error(
        "PROVIDER_OPEN_FAILED",
        "open_provider",
        "The default browser could not open the provider.",
    ))
    .expect("serialize error");
    for value in [request_json, result_json, error_json] {
        assert!(value.get("collection_id").is_none());
        assert!(value.get("collection_revision").is_none());
        assert!(value.get("final_target_root").is_none());
    }
}
