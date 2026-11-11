# pdfs-rs

PDF/A conformance checking for SHAPTH.

`pdfs-rs` is a thin, owned façade over the [`page_validation`](https://crates.io/crates/page_validation)
engine. It exists so that SHAPTH's file-integrity gate can answer three questions the
raw engine does not answer consistently for us:

- which ISO 19005 rule failed (`ISO 19005-1:2005:6.7.2:2`), in a stable report shape;
- one finding per root cause — derived failures (XMP/schema/colour checks that follow
  from an unreadable metadata stream) are not reported separately;
- the verdict as a service contract (`pdfservice.Pdf/Validate`), so the Java/PDFBox
  implementation in `pdf-integrity` can be replaced without touching its callers.

## Legal

Licensed under `MIT` license.

## Features

- PDF/A-1b conformance check over in-memory PDF bytes
- ISO-notation rule ids plus readable messages
- no runtime dependencies (no JVM, no shelling out)

## Usage

Add dependency to Cargo.toml:

```toml
[dependencies]
pdfa = "0.1"
```

```rust
use pdfa::{validate_bytes, Profile};

let report = validate_bytes(&pdf_bytes, Profile::PdfA1b)?;
if !report.conformant {
    for failure in &report.failures {
        eprintln!("{}: {}", failure.rule_id, failure.message);
    }
}
```

## Tests

```bash
cargo test
```

The fixtures under `crates/pdfa/tests/fixtures` are public samples — see the README
there. SHAPTH customer documents must never be committed.
