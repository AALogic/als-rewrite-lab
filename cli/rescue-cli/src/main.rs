use clap::{Parser, Subcommand};
use rescue_analyzer::{
    assess_dependencies, build_preflight_report, discover_project, ProjectDiscoveryRequest,
};
use rescue_core::{
    analyze_als, extract_dependencies, observe_dependency_paths, ALSError, PathObservationContext,
};
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "rescue")]
#[command(about = "Ableton dependency safety CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Analyze { path: PathBuf },
    Extract { path: PathBuf },
    Preflight { path: PathBuf },
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
    }
}

fn run_preflight(path: PathBuf) -> ExitCode {
    let discovery = discover_project(&ProjectDiscoveryRequest {
        source_als_path: path.clone(),
    });
    if !discovery.errors.is_empty() {
        return print_json_with_domain_status(&discovery, false);
    }
    let analysis = match analyze_als(path) {
        Ok(analysis) => analysis,
        Err(error) => return print_als_error(error),
    };
    let extraction = extract_dependencies(&analysis);
    let observations = observe_dependency_paths(
        &extraction,
        &PathObservationContext {
            host_platform: current_host_platform().to_string(),
            confirmed_project_root: discovery.confirmed_project_root.clone(),
            project_root_basis: discovery
                .confirmed_project_root
                .as_ref()
                .map(|_| "confirmed_ableton_project_structure".to_string()),
        },
    );
    let assessment = assess_dependencies(&extraction, &observations);
    let report = build_preflight_report(&discovery, &assessment);
    let success = report.errors.is_empty();
    print_json_with_domain_status(&report, success)
}

fn current_host_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "posix"
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
}
