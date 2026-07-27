use rescue_pipeline::{run_laboratory_package, LaboratoryPackageRequest};
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
}

pub(crate) fn run(arguments: Arguments) -> ExitCode {
    let result = run_laboratory_package(&LaboratoryPackageRequest {
        run_id: arguments.run_id,
        source_als_path: arguments.path,
        scan_roots: arguments.scan_roots,
        max_scan_entries: arguments.max_entries,
        staging_root: arguments.staging_root,
        target_project_root: arguments.target_root,
        private_ledger_path: arguments.private_ledger,
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
