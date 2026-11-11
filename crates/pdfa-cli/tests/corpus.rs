//! Corpus harness: every fixture in `corpus/manifest.json` must produce exactly the
//! expected verdict and rule ids through the CLI.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

const CORPUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus");

#[derive(Deserialize)]
struct Manifest {
    profile: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    file: String,
    #[serde(default)]
    policy: Option<String>,
    conformant: bool,
    failures: Vec<String>,
}

#[derive(Deserialize)]
struct CliReport {
    conformant: bool,
    failures: Vec<CliFailure>,
}

#[derive(Deserialize)]
struct CliFailure {
    rule_id: String,
}

fn manifest() -> Manifest {
    let raw = std::fs::read_to_string(Path::new(CORPUS).join("manifest.json"))
        .expect("corpus/manifest.json");
    serde_json::from_str(&raw).expect("valid manifest")
}

fn run(case: &Case, profile: &str) -> Result<(), String> {
    let path: PathBuf = Path::new(CORPUS).join(&case.file);
    let mut command = Command::new(env!("CARGO_BIN_EXE_pdfs"));
    command.args([
        "check",
        path.to_str().expect("utf-8 path"),
        "--profile",
        profile,
        "--format",
        "json",
    ]);
    if let Some(policy) = &case.policy {
        command.args(["--policy", policy]);
    }
    let output = command.output().expect("run pdfs");

    let expected_code = i32::from(!case.conformant);
    if output.status.code() != Some(expected_code) {
        return Err(format!(
            "{}: exit {:?}, expected {expected_code}",
            case.file, output.status
        ));
    }

    let report: CliReport =
        serde_json::from_slice(&output.stdout).map_err(|err| format!("{}: {err}", case.file))?;

    if report.conformant != case.conformant {
        return Err(format!(
            "{}: conformant={}, expected {}",
            case.file, report.conformant, case.conformant
        ));
    }

    let actual: Vec<&str> = report
        .failures
        .iter()
        .map(|failure| failure.rule_id.as_str())
        .collect();
    let expected: Vec<&str> = case.failures.iter().map(String::as_str).collect();

    if actual != expected {
        return Err(format!("{}: {actual:?}, expected {expected:?}", case.file));
    }

    Ok(())
}

fn run_all(manifest: &Manifest) -> Result<(), String> {
    for case in &manifest.cases {
        run(case, &manifest.profile)?;
    }
    Ok(())
}

#[test]
fn corpus_matches_expectations() {
    let manifest = manifest();
    assert!(
        manifest.cases.iter().any(|case| case.conformant),
        "corpus needs a conformant case"
    );
    assert!(
        manifest.cases.iter().any(|case| !case.conformant),
        "corpus needs a broken case as negative control"
    );

    run_all(&manifest).expect("corpus verdicts");
}

#[test]
fn harness_detects_a_wrong_expectation() {
    let mut manifest = manifest();
    let case = manifest
        .cases
        .iter_mut()
        .find(|case| !case.conformant)
        .expect("a broken case");

    case.failures.clear();

    assert!(
        run_all(&manifest).is_err(),
        "an empty expectation for a broken fixture must fail the harness"
    );
}

#[test]
fn cli_matches_the_library() {
    let library_profile = pdfa::Profile::PdfA1b;

    for case in manifest().cases {
        if case.policy.is_some() {
            continue; // non-default policy, covered by the corpus run
        }
        let path: PathBuf = Path::new(CORPUS).join(&case.file);
        let bytes = std::fs::read(&path).expect("fixture");
        let library = pdfa::validate_bytes(&bytes, library_profile).expect("validate");

        let output = Command::new(env!("CARGO_BIN_EXE_pdfs"))
            .args([
                "check",
                path.to_str().expect("utf-8 path"),
                "--profile",
                "1b",
                "--format",
                "json",
            ])
            .output()
            .expect("run pdfs");

        let cli: CliReport = serde_json::from_slice(&output.stdout).expect("json report");
        let library_rules: Vec<&str> = library
            .failures
            .iter()
            .map(|failure| failure.rule_id.as_str())
            .collect();
        let cli_rules: Vec<&str> = cli
            .failures
            .iter()
            .map(|failure| failure.rule_id.as_str())
            .collect();

        assert_eq!(cli.conformant, library.conformant, "{}", case.file);
        assert_eq!(cli_rules, library_rules, "{}", case.file);
    }
}
