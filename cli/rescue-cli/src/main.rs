use clap::{Parser, Subcommand};
use rescue_application::{
    analyze_project, prepare_copy, DesktopAnalyzeRequest, DesktopPrepareCopyRequest,
};
use rescue_core::{analyze_als, extract_dependencies, ALSError};
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;
use std::process::ExitCode;

mod laboratory_command;

#[derive(Debug, Parser)]
#[command(name = "rescue")]
#[command(about = "Ableton dependency safety CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Analyze {
        path: PathBuf,
    },
    Extract {
        path: PathBuf,
    },
    Preflight {
        path: PathBuf,
    },
    #[command(name = "copy-preview")]
    CopyPreview {
        path: PathBuf,
        #[arg(long)]
        target_root: PathBuf,
    },
    #[command(name = "lab-package")]
    LabPackage {
        path: PathBuf,
        #[arg(long)]
        run_id: String,
        #[arg(long = "scan-root", required = true)]
        scan_roots: Vec<PathBuf>,
        #[arg(long, default_value_t = 100_000)]
        max_entries: usize,
        #[arg(long)]
        staging_root: PathBuf,
        #[arg(long)]
        target_root: PathBuf,
        #[arg(long)]
        private_ledger: PathBuf,
        #[arg(long)]
        selection_file: Option<PathBuf>,
        #[arg(long, required = true)]
        laboratory_write: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Analyze { path } => match analyze_als(path) {
            Ok(analysis) => print_stdout_json(&analysis),
            Err(error) => print_als_error(error),
        },
        Command::Extract { path } => match analyze_als(path) {
            Ok(analysis) => {
                let dependencies = extract_dependencies(&analysis);
                let exit_code = if dependencies.errors.is_empty() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                };
                match print_stdout_json(&dependencies) {
                    ExitCode::SUCCESS => exit_code,
                    failure => failure,
                }
            }
            Err(error) => print_als_error(error),
        },
        Command::Preflight { path } => run_preflight(path),
        Command::CopyPreview { path, target_root } => run_copy_preview(path, target_root),
        Command::LabPackage {
            path,
            run_id,
            scan_roots,
            max_entries,
            staging_root,
            target_root,
            private_ledger,
            selection_file,
            laboratory_write: _,
        } => laboratory_command::run(laboratory_command::Arguments {
            path,
            run_id,
            scan_roots,
            max_entries,
            staging_root,
            target_root,
            private_ledger,
            selection_file,
        }),
    }
}

fn run_copy_preview(path: PathBuf, target_root: PathBuf) -> ExitCode {
    let preview = prepare_copy(&DesktopPrepareCopyRequest {
        request_id: "cli-copy-preview".to_string(),
        source_als_path: path,
        target_project_root: target_root,
    });
    print_json_with_domain_status(&preview, preview.errors.is_empty())
}

fn run_preflight(path: PathBuf) -> ExitCode {
    let result = analyze_project(&DesktopAnalyzeRequest {
        request_id: "cli-preflight".to_string(),
        source_als_path: path,
    });
    match result.preflight_report {
        Some(report) => print_json_with_domain_status(&report, result.errors.is_empty()),
        None => print_json_with_domain_status(&result, false),
    }
}

fn print_json_with_domain_status(value: &impl Serialize, success: bool) -> ExitCode {
    match print_stdout_json(value) {
        ExitCode::SUCCESS if success => ExitCode::SUCCESS,
        ExitCode::SUCCESS => ExitCode::FAILURE,
        failure => failure,
    }
}

fn print_stdout_json(value: &impl Serialize) -> ExitCode {
    match serde_json::to_string_pretty(value) {
        Ok(body) => {
            println!("{body}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!(
                "{}",
                fallback_json_error("CLI_JSON_SERIALIZATION_FAILED", &error.to_string())
            );
            ExitCode::FAILURE
        }
    }
}

fn print_als_error(error: ALSError) -> ExitCode {
    let body = json!({
        "errors": [error.to_info()],
        "warnings": [],
    });

    match serde_json::to_string_pretty(&body) {
        Ok(body) => eprintln!("{body}"),
        Err(error) => eprintln!(
            "{}",
            fallback_json_error("CLI_FATAL_ERROR_SERIALIZATION_FAILED", &error.to_string())
        ),
    }

    ExitCode::FAILURE
}

fn fallback_json_error(code: &str, message: &str) -> String {
    json!({
        "errors": [{
            "error_code": code,
            "message": message,
            "path": null,
        }],
        "warnings": [],
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::error::ErrorKind;
    use clap::Parser;

    #[test]
    fn analyze_uses_json_without_a_redundant_output_flag() {
        assert!(Cli::try_parse_from(["rescue", "analyze", "fixture.als"]).is_ok());

        let error = Cli::try_parse_from(["rescue", "analyze", "fixture.als", "--json"])
            .expect_err("--json should not pretend to select an output mode");
        assert_eq!(error.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn extract_uses_json_without_a_redundant_output_flag() {
        assert!(Cli::try_parse_from(["rescue", "extract", "fixture.als"]).is_ok());

        let error = Cli::try_parse_from(["rescue", "extract", "fixture.als", "--json"])
            .expect_err("--json should not pretend to select an output mode");
        assert_eq!(error.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn preflight_cli_command_is_available() {
        assert!(Cli::try_parse_from(["rescue", "preflight", "fixture.als"]).is_ok());

        let error = Cli::try_parse_from(["rescue", "preflight", "fixture.als", "--json"])
            .expect_err("--json should not pretend to select an output mode");
        assert_eq!(error.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn copy_preview_cli_command_is_available() {
        assert!(Cli::try_parse_from([
            "rescue",
            "copy-preview",
            "fixture.als",
            "--target-root",
            "/tmp/fixture Rescue Project",
        ])
        .is_ok());
    }

    fn laboratory_args() -> [&'static str; 16] {
        [
            "rescue",
            "lab-package",
            "/lab/source.als",
            "--run-id",
            "run-001",
            "--scan-root",
            "/lab/audio",
            "--max-entries",
            "1000",
            "--staging-root",
            "/lab/out.staging",
            "--target-root",
            "/lab/out",
            "--private-ledger",
            "/lab/evidence/run.json",
            "--laboratory-write",
        ]
    }

    #[test]
    fn laboratory_command_requires_explicit_write_flag() {
        let mut args = laboratory_args().to_vec();
        args.pop();
        let error = Cli::try_parse_from(args).expect_err("write flag must be explicit");
        assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn laboratory_command_accepts_bounded_inputs() {
        assert!(Cli::try_parse_from(laboratory_args()).is_ok());
    }

    #[test]
    fn laboratory_command_accepts_optional_selection_file() {
        let mut args = laboratory_args().to_vec();
        let Some(write_flag) = args.pop() else {
            panic!("laboratory fixture must contain the write flag");
        };
        args.extend(["--selection-file", "/lab/selection.json", write_flag]);

        assert!(Cli::try_parse_from(args).is_ok());
    }
}
