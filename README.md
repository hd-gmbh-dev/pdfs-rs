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
pdfs check Pruefbericht.pdf --policy strict                  # literal ISO 19005-1 reading
```

```
Pruefbericht.pdf — PDF/A-1b: non-conformant (1 failed rule)
  ISO 19005-1:2005:6.7.2:2  the catalog metadata stream dictionary contains a Filter key
```

## Service

`crates/pdfa-service` implements `pdfservice.Pdf/Validate` — the contract
`file-integrity` calls today (`shapth-file-integrity` → `pdf_integrity_url`). The
reply carries the validator's findings: `ValidateError.code` is the ISO rule id,
`ValidateError.details` the readable message. Unreadable input answers with
`INVALID_ARGUMENT` instead of a verdict.

```bash
cargo run -p pdfa-service            # listens on $PDFS_ADDR, default 0.0.0.0:11242
```

Smoke test with the *consumer's* proto, i.e. exactly what `file-integrity` speaks:

```bash
python3 -c "import base64,json;open('/tmp/req.json','w').write(json.dumps({'file':base64.b64encode(open('corpus/empty-metadata-filter.pdf','rb').read()).decode()}))"

grpcurl -plaintext \
  -import-path ../shapth/file-integrity/crates/ctx/proto -proto pdf.proto \
  -d @ localhost:11242 pdfservice.Pdf/Validate < /tmp/req.json
# { "error": [ { "code": "ISO 19005-1:2005:6.7.2:2",
#                "details": "the catalog metadata stream dictionary contains a Filter key" } ] }
```

## Container

```bash
docker build -t pdfs-service:local .
```

The image serves gRPC on 11242 and exposes the standard gRPC health service
(`tonic-health`), so Kubernetes `grpc` probes work without an HTTP side-channel.
`helm-charts/skaffold/configs/pdfs/` in the SHAPTH repo runs it in place of the
Java `pdf-integrity` (same contract, ClusterIP-only, gRPC probes).

### Policy

ISO 19005-1 6.7.2 forbids the `Filter` key in the catalog metadata stream dictionary
— including an empty array. veraPDF normalises an empty array to "no filter" and
passes such files. `Policy::Strict` follows the clause,
`Policy::TolerateEmptyFilterArray` follows veraPDF. A non-empty filter — the case
that actually hides the XMP from simple tools — fails under both.

Default is `TolerateEmptyFilterArray` (SHAPTH2-1364 decision, 2026-11-11): the
SHAPTH gate accepts what veraPDF accepts. `--policy strict` / `Policy::Strict` is
the literal-clause mode for audits.

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
