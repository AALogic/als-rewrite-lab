use clap::{Parser, Subcommand};
use rescue_core::{analyze_als, extract_dependencies, ALSError};
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
}
