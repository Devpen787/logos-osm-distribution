# ADR 0002 — Geofabrik source version semantics

Status: accepted for M03/M06

## Context

LP-0018 requires region discovery to show version information and later requires an update check that compares central source versions with registry entries by region path. The Geofabrik JSON index provides region identity and download URLs but does not expose a dedicated snapshot version field.

The OSM PBF header can expose `osmosis_replication_timestamp`, which is strong snapshot metadata tied to the downloaded PBF, but obtaining it requires downloading and opening the PBF. That is too expensive for routine discovery/update checks across the frozen region set.

The LP-0018 specification review discussion does not define a required version encoding, leaving the implementation choice to the challenger.

## Decision

Use two explicit timestamps with different responsibilities:

1. **Registry/source `version`** — the Geofabrik PBF HTTP `Last-Modified` value, normalized before registry storage. This is the cheap source version used for discovery and update comparisons.
2. **`osm_replication_timestamp`** — optional auxiliary metadata read from the downloaded PBF header. This records the OSM replication timestamp contained in the snapshot and is evidence tied to the downloaded bytes.

The published Geofabrik MD5 remains the integrity value for import/source verification. It is not overloaded as a human-facing version.

## Why

- An HTTP metadata request can compare versions without downloading every PBF.
- The PBF replication timestamp provides a second, content-level observation after a snapshot has actually been fetched.
- Keeping the values separate avoids claiming that HTTP publication time and OSM replication time are necessarily identical.
- The MD5 continues to answer a different question: whether the exact bytes match the canonical Geofabrik snapshot.

## Evidence model

A hosted snapshot should eventually preserve at least:

```text
region_path
source_url
version                         # normalized HTTP Last-Modified
checksum                        # published Geofabrik MD5
osm_replication_timestamp       # optional PBF header metadata
cid
registry_timestamp
```

Only `version` is required by LP-0018's minimum registry schema; the replication timestamp is additional metadata.

## Revisit trigger

Revisit if Geofabrik introduces a stable machine-readable version field, or if Logos evaluators publish a more specific interpretation of `version`.
