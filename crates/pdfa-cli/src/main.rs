//! `pdfs` — PDF/A conformance checking from the shell.
//!
//! Exit codes: `0` conformant, `1` non-conformant, `2` the file could not be read.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use pdfa::{validate_bytes_with, Failure, Policy, Profile, Report};

#[derive(Parser)]
#[command(name = "pdfs", version, about = "PDF/A conformance checking")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate a PDF file
    Check {
        /// PDF file to validate
        file: PathBuf,
        /// Conformance profile
        #[arg(long, value_enum, default_value_t = ProfileArg::A1b)]
        profile: ProfileArg,
        /// Output format
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
        /// How to treat an empty /Filter array in the metadata stream dictionary
        #[arg(long, value_enum, default_value_t = PolicyArg::TolerateEmptyFilter)]
        policy: PolicyArg,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ProfileArg {
    #[value(name = "1b")]
    A1b,
}

impl From<ProfileArg> for Profile {
    fn from(value: ProfileArg) -> Self {
        match value {
            ProfileArg::A1b => Profile::PdfA1b,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum PolicyArg {
    /// Report the empty array (ISO 19005-1 6.7.2 wording)
    Strict,
    /// Accept it like veraPDF does; a non-empty filter still fails
    TolerateEmptyFilter,
}

impl From<PolicyArg> for Policy {
    fn from(value: PolicyArg) -> Self {
        match value {
            PolicyArg::Strict => Policy::Strict,
            PolicyArg::TolerateEmptyFilter => Policy::TolerateEmptyFilterArray,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Check {
            file,
            profile,
            format,
            policy,
        } => check(&file, profile.into(), format, policy.into()),
    }
}

fn check(file: &Path, profile: Profile, format: Format, policy: Policy) -> ExitCode {
    let bytes = match std::fs::read(file) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("pdfs: {}: {err}", file.display());
            return ExitCode::from(2);
        }
    };

    let report = match validate_bytes_with(&bytes, profile, policy) {
        Ok(report) => report,
        Err(err) => {
            eprintln!("pdfs: {}: {err}", file.display());
            return ExitCode::from(2);
        }
    };

    match format {
        Format::Text => print_text(file, &report),
        Format::Json => print_json(file, &report),
    }

    if report.conformant {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn print_text(file: &Path, report: &Report) {
    let verdict = if report.conformant {
        "conformant".to_string()
    } else {
        match report.failures.len() {
            1 => "non-conformant (1 failed rule)".to_string(),
            n => format!("non-conformant ({n} failed rules)"),
        }
    };

    println!("{} — {}: {verdict}", file.display(), report.profile);

    for Failure { rule_id, message } in &report.failures {
        println!("  {rule_id}  {message}");
    }
}

#[derive(serde::Serialize)]
struct JsonReport<'a> {
    file: String,
    profile: &'a str,
    conformant: bool,
    tolerated: bool,
    failures: Vec<JsonFailure<'a>>,
}

#[derive(serde::Serialize)]
struct JsonFailure<'a> {
    rule_id: &'a str,
    message: &'a str,
}

fn print_json(file: &Path, report: &Report) {
    let payload = JsonReport {
        file: file.display().to_string(),
        profile: &report.profile.to_string(),
        conformant: report.conformant,
        tolerated: report.tolerated,
        failures: report
            .failures
            .iter()
            .map(|failure| JsonFailure {
                rule_id: &failure.rule_id,
                message: &failure.message,
            })
            .collect(),
    };

    println!(
        "{}",
        serde_json::to_string(&payload).unwrap_or_else(|err| format!("{{\"error\":\"{err}\"}}"))
    );
}
