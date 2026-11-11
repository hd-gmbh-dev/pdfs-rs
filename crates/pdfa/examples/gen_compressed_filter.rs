//! Regenerates `corpus/compressed-metadata-filter.pdf`: the canonical sample with a
//! *non-empty* `/Filter` (FlateDecode) on the metadata stream — a genuine violation
//! that must stay a failure under every policy.
use lopdf::{dictionary, Document, Object, ObjectId, Stream};

fn main() {
    let src = std::env::args().nth(1).expect("src");
    let dst = std::env::args().nth(2).expect("dst");
    let mut doc = Document::load(&src).expect("load");
    let metadata: ObjectId = doc
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.as_dict()
                .or_else(|_| obj.as_stream().map(|s| &s.dict))
                .is_ok_and(|d| {
                    d.get(b"Type")
                        .and_then(Object::as_name)
                        .is_ok_and(|n| n == b"Metadata")
                })
        })
        .map(|(id, _)| *id)
        .expect("metadata object");

    let content = match doc.objects.get(&metadata) {
        Some(Object::Stream(stream)) => stream.content.clone(),
        other => panic!("metadata object is not a stream: {other:?}"),
    };

    let mut compressed = Stream::new(
        dictionary! { "Type" => "Metadata", "Subtype" => "XML", "Filter" => Object::Name(b"FlateDecode".to_vec()) },
        content,
    );
    compressed.compress().expect("compress");
    doc.objects.insert(metadata, Object::Stream(compressed));

    doc.save(&dst).expect("save");
    println!("wrote {dst}");
}
