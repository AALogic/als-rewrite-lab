use rescue_pipeline::{run_laboratory_package, LaboratoryPackageRequest};
use rescue_resolution::UserSelectionSet;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

pub(crate) struct Arguments {
    pub path: PathBuf,
    pub run_id: String,
    pub scan_roots: Vec<PathBuf>,
    pub max_entries: usize,
    pub staging_root: PathBuf,
    pub target_root: PathBuf,
    pub private_ledger: PathBuf,
    pub selection_file: Option<PathBuf>,
}

pub(crate) fn run(arguments: Arguments) -> ExitCode {
    let user_selection_set = match load_selection_file(arguments.selection_file.as_ref()) {
        Ok(value) => value,
        Err(message) => return print_selection_error(&message),
    };
    let result = run_laboratory_package(&LaboratoryPackageRequest {
        run_id: arguments.run_id,
        source_als_path: arguments.path,
        scan_roots: arguments.scan_roots,
        max_scan_entries: arguments.max_entries,
        staging_root: arguments.staging_root,
        target_project_root: arguments.target_root,
        private_ledger_path: arguments.private_ledger,
        user_selection_set,
    });
    let success = result.run_status == "ready_for_manual_ableton_check";
    match serde_json::to_string_pretty(&result) {
        Ok(body) => {
            println!("{body}");
            if success {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!(
                "{{\"errors\":[{{\"error_code\":\"CLI_JSON_SERIALIZATION_FAILED\",\"message\":{}}}]}}",
                serde_json::Value::String(error.to_string())
            );
            ExitCode::FAILURE
        }
    }
}

fn load_selection_file(path: Option<&PathBuf>) -> Result<Option<UserSelectionSet>, String> {
    let Some(path) = path else {
        return Ok(None);
    };
    let bytes =
        fs::read(path).map_err(|error| format!("Cannot read user selection file: {error}"))?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|error| format!("Cannot parse user selection file: {error}"))
}

fn print_selection_error(message: &str) -> ExitCode {
    let body = serde_json::json!({
        "errors": [{
            "error_code": "CLI_USER_SELECTION_FILE_INVALID",
            "message": message,
        }]
    });
    eprintln!("{body}");
    ExitCode::FAILURE
}
