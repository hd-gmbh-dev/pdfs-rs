//! PDF/A conformance checking for SHAPTH.
//!
//! A thin façade over the `page_validation` engine: it fixes the report shape we
//! hand to `pdf-integrity` (ISO rule ids plus a readable message) and keeps the
//! engine replaceable behind one function.

use std::fmt;

use page_validation::{validate_pdf_bytes, ValidationOptions, ValidationProfile};

/// Rule whose failure means the catalog metadata stream is unusable (missing or
/// carrying a `/Filter` key). Everything XMP-derived is then unreliable.
const METADATA_ROOT: &str = "ISO 19005-1:2005:6.7.2:";

/// Rules that only follow from an unreadable metadata stream: the XMP
/// identification/Info-XMP checks, the XMP parse error itself, and the colour
/// checks whose OutputIntent lookup runs through the (unparsable) metadata.
const METADATA_DERIVED: [&str; 5] = [
    "ISO 19005-1:2005:6.7.3:",
    "ISO 19005-1:2005:6.7.9:",
    "ISO 19005-1:2005:6.7.11:",
    "ISO 19005-1:2005:6.2.2:",
    "ISO 19005-1:2005:6.2.3.3:",
];

/// Conformance profile to check against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    PdfA1b,
}

impl Profile {
    fn engine(self) -> ValidationProfile {
        match self {
            Profile::PdfA1b => ValidationProfile::PdfA1b,
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Profile::PdfA1b => f.write_str("PDF/A-1b"),
        }
    }
}

/// One failed rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    /// ISO notation, e.g. `ISO 19005-1:2005:6.7.2:2`.
    pub rule_id: String,
    /// Human readable description of the violation.
    pub message: String,
}

/// Outcome of a validation run.
#[derive(Clone, Debug)]
pub struct Report {
    pub conformant: bool,
    pub profile: Profile,
    pub failures: Vec<Failure>,
}

/// Validates PDF bytes against `profile`.
///
/// # Errors
///
/// Returns the engine error when the input is not a readable PDF or exceeds the
/// engine's safety limits.
pub fn validate_bytes(
    bytes: &[u8],
    profile: Profile,
) -> Result<Report, page_validation::PageError> {
    let options = ValidationOptions::default().profile(profile.engine());
    let report = validate_pdf_bytes(bytes, &options)?;

    Ok(Report {
        conformant: report.is_compliant,
        profile,
        failures: collapse(
            report
                .failures
                .into_iter()
                .map(|failure| Failure {
                    rule_id: failure.rule_id,
                    message: failure.message,
                })
                .collect(),
        ),
    })
}

/// Keeps one finding per root cause: when the metadata stream itself is broken,
/// the XMP/colour rules that fail because of it are dropped.
fn collapse(failures: Vec<Failure>) -> Vec<Failure> {
    let metadata_unusable = failures
        .iter()
        .any(|failure| failure.rule_id.starts_with(METADATA_ROOT));

    if !metadata_unusable {
        return failures;
    }

    failures
        .into_iter()
        .filter(|failure| {
            !METADATA_DERIVED
                .iter()
                .any(|derived| failure.rule_id.starts_with(derived))
        })
        .collect()
}
