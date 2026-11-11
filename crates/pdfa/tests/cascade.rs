//! An unusable metadata stream must be reported once, not as the family of XMP
//! and colour rules that fail because of it.

use lopdf::{Document, Object, ObjectId};
use pdfa::{validate_bytes, Profile};

const CONFORMANT: &[u8] = include_bytes!("fixtures/canonical-pdfa-1b.pdf");

/// Rebuilds the sample with an empty `/Filter` array in the metadata stream
/// dictionary — the upstream construct that trips the cascade.
fn with_empty_metadata_filter() -> Vec<u8> {
    let mut doc = Document::load_mem(CONFORMANT).expect("load fixture");

    let metadata: ObjectId = doc
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.as_dict()
                .or_else(|_| obj.as_stream().map(|stream| &stream.dict))
                .is_ok_and(|dict| {
                    dict.get(b"Type")
                        .and_then(Object::as_name)
                        .is_ok_and(|name| name == b"Metadata")
                })
        })
        .map(|(id, _)| *id)
        .expect("metadata object");

    match doc.objects.get_mut(&metadata) {
        Some(Object::Stream(stream)) => {
            stream.dict.set("Filter", Object::Array(vec![]));
        }
        other => panic!("metadata object is not a stream: {other:?}"),
    }

    let mut out = Vec::new();
    doc.save_to(&mut out).expect("save");
    out
}

#[test]
fn empty_filter_yields_exactly_one_failure() {
    let report = validate_bytes(&with_empty_metadata_filter(), Profile::PdfA1b).expect("readable");

    assert_eq!(
        report
            .failures
            .iter()
            .map(|failure| failure.rule_id.as_str())
            .collect::<Vec<_>>(),
        ["ISO 19005-1:2005:6.7.2:2"],
        "one root cause, no cascade"
    );
}

#[test]
fn missing_metadata_collapses_to_its_root_cause() {
    let report = validate_bytes(include_bytes!("fixtures/no-metadata.pdf"), Profile::PdfA1b)
        .expect("readable");

    assert_eq!(
        report
            .failures
            .iter()
            .map(|failure| failure.rule_id.as_str())
            .collect::<Vec<_>>(),
        ["ISO 19005-1:2005:6.7.2:1"]
    );
}
