# M03 Current Geofabrik Source-Mapping Audit — 2026-09-18

Mission: **M03 — Geofabrik discovery + checksum proof**

Status: `source-confirmed / implementation-patched / final macOS live audit pending`

## Purpose

After the first live `resolve-all` run exposed stale source assumptions, the entire LP-0018 frozen set was cross-checked against the current Geofabrik `index-v1-nogeom.json` rather than fixing mismatches one at a time.

## Source identity findings

Most country and decomposed-country selectors match the expected model directly.

The current Geofabrik index requires these special cases:

### Ireland

LP-0018 describes the country as Ireland, while the current Geofabrik leaf is:

- source id/path: `ireland-and-northern-ireland`
- source parent: `europe`

The implementation uses the actual Geofabrik path as the canonical region key.

### Malaysia

LP-0018 describes the country as Malaysia, while the current Geofabrik leaf is:

- source id/path: `malaysia-singapore-brunei`
- source parent: `asia`

The implementation uses the actual Geofabrik path as the canonical region key.

### United States leaves

LP-0018 registry semantics keep:

- canonical region: `us/<state>`
- registry parent: `us`
- level: `subregion`

But current Geofabrik index identity is:

- source id: `us/<state>`
- source parent: `north-america`

This applies to the eight required leaves:

- `us/california`
- `us/texas`
- `us/florida`
- `us/new-york`
- `us/washington`
- `us/illinois`
- `us/georgia`
- `us/pennsylvania`

The implementation therefore treats registry tree semantics and Geofabrik index selector semantics as separate fields.

## Other decomposed countries

The current Geofabrik index matches the existing selector model for:

- India: source id is the zone leaf (for example `central-zone`), source parent `india`; canonical registry id is `india/<zone>`.
- China: source id is the province leaf, source parent `china`; canonical registry id is `china/<province>`.
- Russia: source id is the federal-district leaf, source parent `russia`; canonical registry id is `russia/<district>`.

## Version semantics

The prize requires version information during discovery and later update comparisons.

Implementation rule:

- source/update `version`: Unix seconds parsed from the final PBF HTTP `Last-Modified` header;
- raw `Last-Modified`, ETag, and final Content-Length are retained as source evidence;
- PBF `osmosis_replication_timestamp` / sequence are auxiliary exact-snapshot metadata after download;
- Geofabrik MD5 remains the byte-integrity proof.

The CLI now exposes:

`geofabrik-check version <index.json> <lp-region-id>`

## Verification state

- current source model cross-checked against the full 72-leaf set;
- deterministic Rust tests updated for special mappings;
- Linux CI green after the mapping + version changes;
- final macOS live `resolve-all` and `version` command remain before F-01 is marked locally verified.
