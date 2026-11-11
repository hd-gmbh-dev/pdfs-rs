# Corpus

Regression fixtures for the façade. `manifest.json` is the expectation: verdict plus
the exact list of rule ids — a run that produces an extra, missing or differently
ordered finding fails.

| file | origin | expected |
| --- | --- | --- |
| `canonical-pdfa-1b.pdf` | `page` test fixtures, MIT (Copyright (c) 2026 Joseph Barbier--Darnal) | conformant |
| `no-metadata.pdf` | derived: catalog `/Metadata 49 0 R` overwritten with spaces (byte-neutral) | fails `ISO 19005-1:2005:6.7.2:1` |
| `empty-metadata-filter.pdf` | derived: metadata stream dictionary gets an empty `/Filter` array (the SHAPTH2-1364 construct) | default (tolerate): conformant; `--policy strict`: fails `ISO 19005-1:2005:6.7.2:2` |
| `compressed-metadata-filter.pdf` | derived: metadata stream really is FlateDecode-compressed (`/Filter [/FlateDecode]`) | fails `ISO 19005-1:2005:6.7.2:2` under **every** policy |

The last two pin the SHAPTH2-1364 policy (decided 2026-11-11): an empty array is
tolerated (what veraPDF does), a non-empty filter never is.

Regenerate the derived fixtures from the canonical sample:

```bash
# empty /Filter array in the metadata stream dictionary
# (compressed variant: cargo run -q -p pdfa --example gen_compressed_filter -- …)
cargo run -q -p pdfa --example gen_empty_filter -- \
  crates/pdfa/tests/fixtures/canonical-pdfa-1b.pdf corpus/empty-metadata-filter.pdf

# missing /Metadata entry in the catalog (byte-neutral patch)
python3 - <<'PY'
d = open('crates/pdfa/tests/fixtures/canonical-pdfa-1b.pdf', 'rb').read()
needle = b'/Metadata 49 0 R'
assert needle in d
open('corpus/no-metadata.pdf', 'wb').write(d.replace(needle, b' ' * len(needle)))
PY
```

SHAPTH customer or JIRA documents never go into this directory.
