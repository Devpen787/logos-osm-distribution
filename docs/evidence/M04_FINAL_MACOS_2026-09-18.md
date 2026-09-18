# M04 Final macOS Storage Verification — 2026-09-18

Mission: **M04 — Logos Storage CID round-trip proof**

Status: `complete / locally-verified`

## Environment

- implementation repository: `Devpen787/logos-osm-distribution`
- branch: `feat/m04-logos-storage-proof`
- proof machine: macOS Apple Silicon
- `logoscore` pin: `ab1ae3509070163c949afdc30f5d5db8898fd5b0`
- Storage module pin: `logos-co/logos-storage-module@bcc29f05e2a7b4a2d215d09640aaa7436359906b`
- Storage module metadata version: `2.1.3`
- underlying Storage runtime reported: `0.4.5-e3225940`

## Input artifact

M03-verified Ethiopia Geofabrik PBF:

- filename: `ethiopia-latest.osm.pbf`
- byte length: `139733363`
- expected MD5: `426bd510159627dc139d4d0ad3bc6acd`
- observed MD5 before Storage: `426bd510159627dc139d4d0ad3bc6acd`

## Storage startup

The isolated Logos daemon reached running state and generated its daemon/client config.

`storage_module` loaded successfully, then:

- `init` returned success;
- `start` returned success;
- Storage debug returned a live local node identity;
- the proof used `no-bootstrap-node=true`, so no remote replication/network availability claim is made.

## Upload proof

The verified Ethiopia PBF was submitted through:

`storage_module.uploadUrl(<absolute-pbf-path>, 65536)`

The Storage manifest recorded:

- CID: `zDvZRwzmDq9DKvNkN9JJhhN81TeQpcRPz3W1Aj4z1KHKqGmQL7vK`
- tree CID: `zDzSvJTf8qBGgS6Ug4g3v8px2ATSzV2MoFdutEyt4HJbG17BGC4X`
- dataset size: `139733363`
- block size: `65536`
- filename: `ethiopia-latest.osm.pbf`

Storage logs reported the dataset stored as 2133 blocks.

`exists(cid)` returned `true`.

## Download proof

The same CID was retrieved through:

`storage_module.downloadToUrl(cid, <destination>, true, 65536)`

The `local=true` flag intentionally restricts this proof to the local Storage node.

Verification results:

- byte-for-byte `cmp`: **PASS**
- retrieved byte length: `139733363`
- expected Geofabrik MD5: `426bd510159627dc139d4d0ad3bc6acd`
- observed retrieved MD5: `426bd510159627dc139d4d0ad3bc6acd`

Result: the exact M03-verified Geofabrik bytes survived:

`verified source bytes -> Logos Storage -> CID -> Storage retrieval -> identical bytes`.

## Shutdown

The proof completed a clean Storage lifecycle:

- `storage_module.stop`: success
- `storage_module.destroy`: success
- `logoscore stop`: success

## Requirement conclusions

This mission proves the **Storage foundations** for LP-0018 but does not complete the full prize criteria:

- F-02 Host workflow: `in_progress`
  - Storage leg locally verified;
  - LEZ registry write still required in M05/M06.
- F-03 Hosted download/fallback: `in_progress`
  - hosted-by-CID local Storage retrieval locally verified;
  - Geofabrik fallback selection still required in M06/M07.
- R-01 transient Storage retry/backoff: `unaddressed`
  - M04 observed the success path only;
  - bounded retry/backoff remains M10.

## Truth boundary

M04 is **locally-verified real Logos Storage**, not:

- remote replication proof;
- Logos testnet verification;
- LEZ registration;
- evaluator adoption coverage.

## Next

Merge PR #2 to `main`, then start M05: a minimal SPEL/LEZ OSM registry with required fields, region/parent/CID queries, timestamp ordering, batch-register API, and generated IDL.
