use serde::{Deserialize, Serialize};

pub(crate) const PROVIDER_ID: &str = "wetransfer_web";
const SCHEMA_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalProviderOpenRequest {
    pub request_id: String,
    pub provider_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalProviderOpenResult {
    pub schema_version: String,
    pub request_id: String,
    pub provider_id: String,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalProviderOpenError {
    pub error_code: String,
    pub stage: String,
    pub message: String,
}

pub(crate) fn validate_request(
    request: &ExternalProviderOpenRequest,
) -> Result<(), ExternalProviderOpenError> {
    if request.request_id.trim().is_empty() {
        return Err(error(
            "PROVIDER_OPEN_REQUEST_INVALID",
            "validate",
            "The provider request identity is missing.",
        ));
    }
    if request.provider_id != PROVIDER_ID {
        return Err(error(
            "PROVIDER_OPEN_UNSUPPORTED",
            "validate",
            "The requested provider is not supported.",
        ));
    }
    Ok(())
}

pub(crate) fn opened(request: &ExternalProviderOpenRequest) -> ExternalProviderOpenResult {
    ExternalProviderOpenResult {
        schema_version: SCHEMA_VERSION.to_string(),
        request_id: request.request_id.clone(),
        provider_id: request.provider_id.clone(),
        state: "opened".to_string(),
    }
}

pub(crate) fn error(code: &str, stage: &str, message: &str) -> ExternalProviderOpenError {
    ExternalProviderOpenError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
#[path = "external_folder_handoff_tests.rs"]
mod tests;
