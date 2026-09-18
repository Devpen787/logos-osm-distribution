# M03 Final macOS Verification — 2026-09-18

Mission: **M03 — Geofabrik discovery + checksum proof**

Status: `complete / locally-verified`

## Final macOS Apple Silicon gate

After pulling the final source-mapping and version-probe changes, the complete M03 verification passed.

### Local tests

- `geofabrik`: 7 passed, 0 failed
- `osm-domain`: 5 passed, 0 failed

The final suite includes regression coverage for:
- fail-closed checksum mismatch;
- current Ireland Geofabrik path;
- current Malaysia combined Geofabrik path;
- U.S. source-parent semantics;
- source-version header parsing;
- exact 72-leaf frozen-set validation.

### Live current-index audit

The current Geofabrik index resolved the entire frozen set:

- resolved region count: **72**
- Ireland mapping: passed
- Malaysia mapping: passed
- all 8 required U.S. mappings: passed

Result: **ALL 72 LP-0018 REGIONS RESOLVED**

### Live version proof

For Ethiopia, the source-version probe returned:

- `version_unix_seconds`: `1789685417`
- `last_modified`: `Thu, 17 Sep 2026 22:50:17 GMT`
- `etag`: `"8542973-65bb59b94857b"`
- `content_length`: `139733363`

This matches the source-version semantics in the M03 ADR: discovery/update version is the final PBF HTTP `Last-Modified` normalized to Unix seconds, while raw HTTP metadata remains evidence.

### Prior real-byte proof retained

The earlier live Ethiopia proof remains part of the M03 evidence packet:

- downloaded real PBF byte length: `139733363`
- published MD5: `426bd510159627dc139d4d0ad3bc6acd`
- observed MD5: `426bd510159627dc139d4d0ad3bc6acd`
- replication timestamp: `1789676465`
- replication sequence: `4905`
- replication base URL: `https://download.geofabrik.de/africa/ethiopia-updates`

## Requirement conclusions

- **F-01 Region discovery:** `locally-verified`
- **R-02 checksum mismatch behavior:** `locally-verified`

M03 makes no Logos Storage, LEZ, or testnet claim.

## Next

Merge PR #1 to `main`, then begin M04: store already-verified snapshot bytes through the current Logos Storage path, retrieve by CID, and compare retrieved bytes/hash to the verified source snapshot.
