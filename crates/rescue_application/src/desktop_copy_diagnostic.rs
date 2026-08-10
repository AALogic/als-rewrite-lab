use crate::{
    DesktopApplicationError, DesktopCopyDiagnosticReport, DesktopDiagnosticError,
    DESKTOP_COPY_DIAGNOSTIC_SCHEMA_VERSION, DESKTOP_COPY_SERVICE_VERSION,
};
use rescue_pipeline::{CurrentPathCopyResult, CURRENT_PATH_COPY_PIPELINE_VERSION};
use std::path::Path;

pub(crate) fn from_pipeline(
    request_id: &str,
    operation_kind: &str,
    result: &CurrentPathCopyResult,
    errors: &[DesktopApplicationError],
    sensitive_paths: &[&Path],
    elapsed_ms: u64,
) -> DesktopCopyDiagnosticReport {
    let source = result.package_plan.as_ref().map(|plan| &plan.source_als);
    let compatibility_status =
        if result.rewrite_policy == rescue_pipeline::COMPATIBILITY_LAB_REWRITE_POLICY {
            if errors.is_empty() {
                "experimental_candidate"
            } else {
                "experimental_blocked"
            }
        } else {
            "confirmed_profile"
        };
    report(
        request_id,
        operation_kind,
        &result.pipeline_version,
        &result.run_status,
        &result.completed_stage,
        Counts {
            required: result.required_asset_count,
            system: result.system_dependency_count,
            copied: result.copied_asset_count,
            rewritten: result.rewritten_reference_count,
            omitted: result.omitted_asset_count,
        },
        errors,
        sensitive_paths,
        &result.rewrite_policy,
        source.and_then(|item| item.ableton_document_version.as_deref()),
        source.and_then(|item| item.ableton_creator_version.as_deref()),
        source.and_then(|item| item.ableton_minor_version.as_deref()),
        compatibility_status,
        elapsed_ms,
    )
}

pub(crate) struct BeforePipelineDiagnostic<'a> {
    pub request_id: &'a str,
    pub operation_kind: &'a str,
    pub run_status: &'a str,
    pub completed_stage: &'a str,
    pub errors: &'a [DesktopApplicationError],
    pub sensitive_paths: &'a [&'a Path],
    pub rewrite_policy: &'a str,
    pub elapsed_ms: u64,
}

pub(crate) fn before_pipeline(input: BeforePipelineDiagnostic<'_>) -> DesktopCopyDiagnosticReport {
    let BeforePipelineDiagnostic {
        request_id,
        operation_kind,
        run_status,
        completed_stage,
        errors,
        sensitive_paths,
        rewrite_policy,
        elapsed_ms,
    } = input;
    report(
        request_id,
        operation_kind,
        CURRENT_PATH_COPY_PIPELINE_VERSION,
        run_status,
        completed_stage,
        Counts::default(),
        errors,
        sensitive_paths,
        rewrite_policy,
        None,
        None,
        None,
        "not_evaluated",
        elapsed_ms,
    )
}

#[derive(Default)]
struct Counts {
    required: usize,
    system: usize,
    copied: usize,
    rewritten: usize,
    omitted: usize,
}

#[allow(clippy::too_many_arguments)]
fn report(
    request_id: &str,
    operation_kind: &str,
    pipeline_version: &str,
    run_status: &str,
    completed_stage: &str,
    counts: Counts,
    errors: &[DesktopApplicationError],
    sensitive_paths: &[&Path],
    rewrite_policy: &str,
    ableton_document_version: Option<&str>,
    ableton_creator_version: Option<&str>,
    ableton_minor_version: Option<&str>,
    compatibility_status: &str,
    elapsed_ms: u64,
) -> DesktopCopyDiagnosticReport {
    DesktopCopyDiagnosticReport {
        diagnostic_schema_version: DESKTOP_COPY_DIAGNOSTIC_SCHEMA_VERSION.to_string(),
        request_id: request_id.to_string(),
        operation_kind: operation_kind.to_string(),
        service_version: DESKTOP_COPY_SERVICE_VERSION.to_string(),
        pipeline_version: pipeline_version.to_string(),
        build_commit: option_env!("ALS_RESCUE_BUILD_COMMIT")
            .unwrap_or("development")
            .to_string(),
        host_os: std::env::consts::OS.to_string(),
        host_arch: std::env::consts::ARCH.to_string(),
        rewrite_policy: rewrite_policy.to_string(),
        ableton_document_version: safe_metadata_value(ableton_document_version),
        ableton_creator_version: safe_metadata_value(ableton_creator_version),
        ableton_minor_version: safe_metadata_value(ableton_minor_version),
        compatibility_status: compatibility_status.to_string(),
        run_status: run_status.to_string(),
        completed_stage: completed_stage.to_string(),
        elapsed_ms,
        required_asset_count: counts.required,
        system_dependency_count: counts.system,
        copied_asset_count: counts.copied,
        rewritten_reference_count: counts.rewritten,
        omitted_asset_count: counts.omitted,
        errors: errors
            .iter()
            .map(|item| DesktopDiagnosticError {
                error_code: item.error_code.clone(),
                stage: item.stage.clone(),
                message: redact_message(&item.message, sensitive_paths),
            })
            .collect(),
    }
}

fn safe_metadata_value(value: Option<&str>) -> Option<String> {
    value
        .filter(|item| item.len() <= 120 && !contains_local_file_detail(item))
        .map(ToString::to_string)
}

fn redact_message(message: &str, sensitive_paths: &[&Path]) -> String {
    if contains_local_file_detail(message) {
        return match windows_error_code(message) {
            Some(code) => format!("Local file details redacted; windows_error_code={code}"),
            None => "Local file details redacted".to_string(),
        };
    }
    let mut redacted = message.to_string();
    for path in sensitive_paths {
        redact_value(&mut redacted, &path.to_string_lossy());
        if let Some(name) = path.file_name().and_then(|value| value.to_str()) {
            redact_value(&mut redacted, name);
        }
    }
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        redact_value(&mut redacted, &home.to_string_lossy());
    }
    redacted
        .split_whitespace()
        .map(redact_path_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_local_file_detail(message: &str) -> bool {
    message.split_whitespace().any(|token| {
        let lower = token.to_ascii_lowercase();
        token.starts_with('/')
            || token.starts_with("\\\\")
            || token.as_bytes().get(1) == Some(&b':')
            || token.contains('/')
            || token.contains('\\')
            || [".als", ".wav", ".wave", ".aif", ".aiff", ".mp3", ".flac"]
                .iter()
                .any(|extension| lower.contains(extension))
    })
}

fn windows_error_code(message: &str) -> Option<&str> {
    let value = message.split("windows_error_code=").nth(1)?;
    let digits = value
        .trim_start()
        .split(|character: char| !character.is_ascii_digit())
        .next()
        .unwrap_or("");
    (!digits.is_empty()).then_some(digits)
}

fn redact_value(message: &mut String, value: &str) {
    if !value.is_empty() {
        *message = message.replace(value, "[redacted]");
    }
}

fn redact_path_token(token: &str) -> &str {
    let lower = token.to_ascii_lowercase();
    let looks_like_path = token.starts_with('/')
        || token.starts_with("\\\\")
        || token.as_bytes().get(1) == Some(&b':')
        || [".als", ".wav", ".aif", ".aiff", ".mp3", ".flac"]
            .iter()
            .any(|extension| lower.contains(extension));
    if looks_like_path {
        "[redacted]"
    } else {
        token
    }
}

#[cfg(test)]
mod tests {
    use super::redact_message;
    use std::path::Path;

    #[test]
    fn redacts_known_paths_and_media_tokens() {
        let source = Path::new("/Users/private/Secret Set.als");
        let target = Path::new("/Users/private/Secret Rescue Project");
        let message = "Cannot use /Users/private/Secret Set.als or hidden.wav at C:\\Private\\x";
        let redacted = redact_message(message, &[source, target]);

        assert!(!redacted.contains("/Users/private"));
        assert!(!redacted.contains("Secret Set.als"));
        assert!(!redacted.contains("hidden.wav"));
        assert!(!redacted.contains("C:\\Private"));
    }

    #[test]
    fn redacts_unlisted_audio_path_with_spaces_and_keeps_windows_error_code() {
        let message =
            "Cannot copy C:\\Private Artist\\Secret Project\\Kick One.wav; windows_error_code=32";
        let redacted = redact_message(message, &[]);

        assert_eq!(
            redacted,
            "Local file details redacted; windows_error_code=32"
        );
        assert!(!redacted.contains("Private Artist"));
        assert!(!redacted.contains("Secret Project"));
        assert!(!redacted.contains("Kick One"));
    }
}
