//! Smoke tests for the wrapper: one conformant sample, one deliberately broken.

use pdfa::{validate_bytes, Profile};

const CONFORMANT: &[u8] = include_bytes!("fixtures/canonical-pdfa-1b.pdf");
const BROKEN: &[u8] = include_bytes!("fixtures/no-metadata.pdf");

#[test]
fn conformant_sample_passes() {
    let report = validate_bytes(CONFORMANT, Profile::PdfA1b).expect("valid pdf");

    assert!(report.conformant, "failures: {:?}", report.failures);
    assert_eq!(report.profile, Profile::PdfA1b);
    assert!(report.failures.is_empty(), "{:?}", report.failures);
}

#[test]
fn broken_sample_reports_iso_rule_ids() {
    let report = validate_bytes(BROKEN, Profile::PdfA1b).expect("readable pdf");

    assert!(!report.conformant);
    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.rule_id == "ISO 19005-1:2005:6.7.2:1"),
        "rule ids: {:?}",
        report
            .failures
            .iter()
            .map(|failure| &failure.rule_id)
            .collect::<Vec<_>>()
    );
}

#[test]
fn garbage_input_is_an_error_not_a_verdict() {
    assert!(validate_bytes(b"not a pdf at all", Profile::PdfA1b).is_err());
}
