use serde::{Deserialize, Serialize};

const SCHEMA_VERSION: &str = "0.1";
#[cfg(test)]
pub(crate) const DRAG_OPERATION_COPY_ONLY: &str = "copy";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniversalPayloadDragRequest {
    pub request_id: String,
    pub collection_id: String,
    pub collection_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniversalPayloadDragPrepared {
    pub schema_version: String,
    pub attempt_id: String,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniversalPayloadDragFinished {
    pub schema_version: String,
    pub attempt_id: String,
    pub outcome: String,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniversalPayloadDragError {
    pub error_code: String,
    pub stage: String,
    pub message: String,
}

impl From<crate::transfer_payload::TransferPayloadError> for UniversalPayloadDragError {
    fn from(error: crate::transfer_payload::TransferPayloadError) -> Self {
        Self {
            error_code: error.error_code,
            stage: error.stage,
            message: error.message,
        }
    }
}

pub(crate) fn validate_request(
    request: &UniversalPayloadDragRequest,
) -> Result<(), UniversalPayloadDragError> {
    if request.request_id.trim().is_empty()
        || request.collection_id.trim().is_empty()
        || request.collection_revision == 0
    {
        return Err(error(
            "PAYLOAD_DRAG_REQUEST_INVALID",
            "validate",
            "The payload drag request is incomplete.",
        ));
    }
    Ok(())
}

pub(crate) fn prepared(attempt_id: String) -> UniversalPayloadDragPrepared {
    UniversalPayloadDragPrepared {
        schema_version: SCHEMA_VERSION.to_string(),
        attempt_id,
        state: "armed".to_string(),
    }
}

pub(crate) fn finished(
    attempt_id: &str,
    outcome: &str,
    error_code: Option<&str>,
) -> UniversalPayloadDragFinished {
    UniversalPayloadDragFinished {
        schema_version: SCHEMA_VERSION.to_string(),
        attempt_id: attempt_id.to_string(),
        outcome: outcome.to_string(),
        error_code: error_code.map(str::to_string),
    }
}

pub(crate) fn error(code: &str, stage: &str, message: &str) -> UniversalPayloadDragError {
    UniversalPayloadDragError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn universal_drag_operation_is_copy_only() {
        assert_eq!(DRAG_OPERATION_COPY_ONLY, "copy");
    }

    #[test]
    fn universal_drag_wire_contract_is_path_free() {
        let request = UniversalPayloadDragRequest {
            request_id: "drag-1".to_string(),
            collection_id: "collection-1".to_string(),
            collection_revision: 2,
        };
        let request_value = serde_json::to_value(request);
        let prepared_value = serde_json::to_value(prepared("attempt-1".to_string()));
        let finished_value = serde_json::to_value(finished("attempt-1", "cancelled", None));
        assert_eq!(
            request_value
                .ok()
                .and_then(|value| value.as_object().map(|map| map.len())),
            Some(3)
        );
        assert_eq!(
            prepared_value
                .ok()
                .and_then(|value| value.as_object().map(|map| map.len())),
            Some(3)
        );
        assert_eq!(
            finished_value
                .ok()
                .and_then(|value| value.as_object().map(|map| map.len())),
            Some(4)
        );
    }
}
