# pdfs-rs

PDF/A conformance checking in Rust.

`pdfs-rs` is a thin wrapper over the [`page_validation`](https://crates.io/crates/page_validation)
engine. It exists because the raw engine answers three questions we need answered
consistently:

- which ISO 19005 rule failed (`ISO 19005-1:2005:6.7.2:2`), in a stable report shape;
- one finding per root cause — derived failures (XMP/schema/colour checks that follow
  from an unreadable metadata stream) are not reported separately;
- the verdict as a service contract (`pdfservice.Pdf/Validate`), so an existing
  Java/PDFBox implementation can be replaced without touching its callers.

## Legal

Licensed under `MIT` license.

## Features

- PDF/A-1b conformance check over in-memory PDF bytes
- ISO-notation rule ids plus readable messages
- one finding per root cause, no cascade
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
pdfs check report.pdf                      # exit 0 conformant, 1 non-conformant, 2 unreadable
pdfs check report.pdf --format json        # machine readable report
pdfs check report.pdf --policy strict      # literal ISO 19005-1 reading
```

```
report.pdf — PDF/A-1b: non-conformant (1 failed rule)
  ISO 19005-1:2005:6.7.2:2  the catalog metadata stream dictionary contains a Filter key
```

## Service

`crates/pdfa-service` implements `pdfservice.Pdf/Validate`, a small contract that
already exists in the wild (see `crates/pdfa-service/proto/pdf.proto`). The reply
carries the validator's findings: `ValidateError.code` is the ISO rule id,
`ValidateError.details` the readable message. Unreadable input answers with
`INVALID_ARGUMENT` instead of a verdict.

```bash
cargo run -p pdfa-service            # listens on $PDFS_ADDR, default 0.0.0.0:11242
```

Smoke test with the proto as shipped here:

```bash
python3 -c "import base64,json;open('/tmp/req.json','w').write(json.dumps({'file':base64.b64encode(open('corpus/empty-metadata-filter.pdf','rb').read()).decode()}))"

grpcurl -plaintext \
  -import-path crates/pdfa-service/proto -proto pdf.proto \
  -d @ localhost:11242 pdfservice.Pdf/Validate < /tmp/req.json
# { "valid": true }
```

## Container

```bash
docker build -t pdfs-service:local .
```

The image serves gRPC on 11242 and exposes the standard gRPC health service
(`tonic-health`), so Kubernetes `grpc` probes work without an HTTP side-channel.
The build images default to public ones and can be overridden:

```bash
docker build --build-arg RUST_IMAGE=rust:1.89-alpine --build-arg BASE_IMAGE=alpine:3.24 .
```

## Policy

ISO 19005-1 6.7.2 forbids the `Filter` key in the catalog metadata stream dictionary
— including an empty array. veraPDF normalises an empty array to "no filter" and
passes such files. `Policy::Strict` follows the clause,
`Policy::TolerateEmptyFilterArray` follows veraPDF. A non-empty filter — the case
that actually hides the XMP from simple tools — fails under both.

Default is `TolerateEmptyFilterArray`: a gate built on this crate accepts what
veraPDF accepts. `--policy strict` / `Policy::Strict` is the literal-clause mode for
audits.

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

The fixtures under `crates/pdfa/tests/fixtures` and `corpus/` are public samples with
their origin noted — see the READMEs there. Never commit customer or ticket
documents.
