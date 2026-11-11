use lopdf::{Document, Object, ObjectId};

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
    if let Some(Object::Stream(stream)) = doc.objects.get_mut(&metadata) {
        stream.dict.set("Filter", Object::Array(vec![]));
    }
    doc.save(&dst).expect("save");
    println!("wrote {dst}");
}
