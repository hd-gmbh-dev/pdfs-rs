use pdfa::{validate_bytes_with, Policy, Profile};
fn main() {
    for p in std::env::args().skip(1) {
        let bytes = std::fs::read(&p).expect("read");
        for policy in [Policy::Strict, Policy::TolerateEmptyFilterArray] {
            let r = validate_bytes_with(&bytes, Profile::PdfA1b, policy).expect("validate");
            println!(
                "{p} [{policy:?}] conformant={} tolerated={} failures={:?}",
                r.conformant,
                r.tolerated,
                r.failures.iter().map(|f| &f.rule_id).collect::<Vec<_>>()
            );
        }
    }
}
