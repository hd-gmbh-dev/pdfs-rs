//! The empty-`/Filter`-array policy: tolerating it must never tolerate a real filter.

use pdfa::{validate_bytes, validate_bytes_with, Policy, Profile};

const CONFORMANT: &[u8] = include_bytes!("fixtures/canonical-pdfa-1b.pdf");
const EMPTY_FILTER: &[u8] = include_bytes!("../../../corpus/empty-metadata-filter.pdf");
const COMPRESSED_FILTER: &[u8] = include_bytes!("../../../corpus/compressed-metadata-filter.pdf");
const NO_METADATA: &[u8] = include_bytes!("fixtures/no-metadata.pdf");

fn rules(bytes: &[u8], policy: Policy) -> (bool, bool, Vec<String>) {
    let report = validate_bytes_with(bytes, Profile::PdfA1b, policy).expect("readable pdf");
    (
        report.conformant,
        report.tolerated,
        report
            .failures
            .iter()
            .map(|failure| failure.rule_id.clone())
            .collect(),
    )
}

#[test]
fn tolerating_an_empty_array_is_the_default() {
    let default = validate_bytes(EMPTY_FILTER, Profile::PdfA1b).expect("readable");

    assert!(default.conformant);
    assert!(default.tolerated);
}

#[test]
fn empty_filter_array_can_be_tolerated() {
    assert_eq!(
        rules(EMPTY_FILTER, Policy::Strict),
        (false, false, vec!["ISO 19005-1:2005:6.7.2:2".to_string()])
    );
    assert_eq!(
        rules(EMPTY_FILTER, Policy::TolerateEmptyFilterArray),
        (true, true, vec![])
    );
}

#[test]
fn a_non_empty_filter_is_never_tolerated() {
    for policy in [Policy::Strict, Policy::TolerateEmptyFilterArray] {
        assert_eq!(
            rules(COMPRESSED_FILTER, policy),
            (false, false, vec!["ISO 19005-1:2005:6.7.2:2".to_string()]),
            "{policy:?}"
        );
    }
}

#[test]
fn tolerated_policy_keeps_other_failures() {
    assert_eq!(
        rules(NO_METADATA, Policy::TolerateEmptyFilterArray),
        (false, false, vec!["ISO 19005-1:2005:6.7.2:1".to_string()])
    );
    assert!(rules(CONFORMANT, Policy::TolerateEmptyFilterArray).0);
}
