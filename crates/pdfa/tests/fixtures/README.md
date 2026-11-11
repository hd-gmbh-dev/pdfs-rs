# Fixtures

- `canonical-pdfa-1b.pdf` — conformant PDF/A-1b sample, copied unchanged from the
  `page` project's test fixtures (`crates/page_validation/tests/fixtures/canonical-pdfa-1b.pdf`,
  MIT License, Copyright (c) 2026 Joseph Barbier--Darnal). The upstream `.typ`
  source sits next to it there, so the sample can be regenerated.
- `no-metadata.pdf` — derived from the file above: the catalog's `/Metadata 49 0 R`
  entry is overwritten with spaces (byte-neutral, cross-reference table untouched),
  which breaks ISO 19005-1 6.7.2 (the catalog must reference a metadata stream).
  Used as the negative control.

Customer or ticket documents never go into this repository.
