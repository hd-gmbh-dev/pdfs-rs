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

## CLI

```bash
cargo install --path crates/pdfa-cli      # installs `pdfs`
pdfs check Pruefbericht.pdf               # exit 0 conformant, 1 non-conformant, 2 unreadable
pdfs check Pruefbericht.pdf --format json  # machine readable report
```

```
Pruefbericht.pdf — PDF/A-1b: non-conformant (1 failed rule)
  ISO 19005-1:2005:6.7.2:2  the catalog metadata stream dictionary contains a Filter key
```

## Corpus

`corpus/` holds public regression fixtures plus `manifest.json` with the expected
verdict and rule ids per file — see [corpus/README.md](corpus/README.md). The
harness (`crates/pdfa-cli/tests/corpus.rs`) runs the CLI over every case, checks
exit code, verdict and the exact rule list, and fails if a broken fixture stops
being detected. It also asserts the CLI agrees with the library for each fixture.

## Tests

```bash
cargo test
```

The fixtures under `crates/pdfa/tests/fixtures` are public samples — see the README
there. SHAPTH customer documents must never be committed.
