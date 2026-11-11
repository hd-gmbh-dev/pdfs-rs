//! PDF/A conformance checking for SHAPTH.
//!
//! A thin façade over the `page_validation` engine: it fixes the report shape we
//! hand to `pdf-integrity` (ISO rule ids plus a readable message) and keeps the
//! engine replaceable behind one function.

use page_validation::{validate_pdf_bytes, ValidationOptions, ValidationProfile};

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
        failures: report
            .failures
            .into_iter()
            .map(|failure| Failure {
                rule_id: failure.rule_id,
                message: failure.message,
            })
            .collect(),
    })
}
