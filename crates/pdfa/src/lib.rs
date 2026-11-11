//! PDF/A conformance checking.
//!
//! A thin wrapper over the `page_validation` engine: it fixes the report shape
//! (ISO rule ids plus a readable message), reports one finding per root cause and
//! keeps the engine replaceable behind one function.

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

/// How the wrapper treats the empty-`/Filter`-array edge case (ISO 19005-1 6.7.2).
///
/// The clause forbids the `Filter` key in the catalog metadata stream dictionary,
/// even an empty array; veraPDF normalises an empty array to "no filter" and passes
/// such files. `Strict` follows the clause, `TolerateEmptyFilterArray` follows
/// veraPDF. A *non-empty* filter is always reported — it really is a violation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Policy {
    Strict,
    #[default]
    TolerateEmptyFilterArray,
}

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
    /// Set when a failure was accepted because of [`Policy::TolerateEmptyFilterArray`].
    pub tolerated: bool,
}

/// Validates PDF bytes against `profile` using the default [`Policy`].
///
/// # Errors
///
/// Returns the engine error when the input is not a readable PDF or exceeds the
/// engine's safety limits.
pub fn validate_bytes(
    bytes: &[u8],
    profile: Profile,
) -> Result<Report, page_validation::PageError> {
    validate_bytes_with(bytes, profile, Policy::default())
}

/// Validates PDF bytes against `profile` and `policy`.
///
/// # Errors
///
/// Returns the engine error when the input is not a readable PDF or exceeds the
/// engine's safety limits.
pub fn validate_bytes_with(
    bytes: &[u8],
    profile: Profile,
    policy: Policy,
) -> Result<Report, page_validation::PageError> {
    let options = ValidationOptions::default().profile(profile.engine());
    let report = validate_pdf_bytes(bytes, &options)?;

    let mut failures = collapse(
        report
            .failures
            .into_iter()
            .map(|failure| Failure {
                rule_id: failure.rule_id,
                message: failure.message,
            })
            .collect(),
    );

    let mut tolerated = false;
    if policy == Policy::TolerateEmptyFilterArray
        && failures.len() == 1
        && failures[0].rule_id == EMPTY_FILTER_RULE
        && metadata_filter_is_empty(bytes)
    {
        failures.clear();
        tolerated = true;
    }

    Ok(Report {
        conformant: failures.is_empty(),
        profile,
        failures,
        tolerated,
    })
}

/// The rule that fires for a `Filter` key in the catalog metadata stream.
const EMPTY_FILTER_RULE: &str = "ISO 19005-1:2005:6.7.2:2";

/// True when the catalog metadata stream dictionary carries a `Filter` entry that
/// names no filter (an empty array), i.e. nothing is actually encoded.
///
/// Any parse problem answers `false`, so tolerance never widens silently.
fn metadata_filter_is_empty(bytes: &[u8]) -> bool {
    use lopdf::Object;

    let Ok(document) = lopdf::Document::load_mem(bytes) else {
        return false;
    };

    document
        .objects
        .values()
        .filter_map(|object| {
            object
                .as_dict()
                .ok()
                .or_else(|| object.as_stream().map(|stream| &stream.dict).ok())
        })
        .filter(|dict| {
            dict.get(b"Type")
                .and_then(Object::as_name)
                .is_ok_and(|name| name == b"Metadata")
        })
        .any(|dict| matches!(dict.get(b"Filter"), Ok(Object::Array(items)) if items.is_empty()))
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
