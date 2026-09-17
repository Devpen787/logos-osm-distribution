# Architecture

## Objective

Build LP-0018 as a set of independently testable components around one canonical data flow:

```text
Geofabrik source
  -> region metadata
  -> PBF bytes
  -> checksum verification
  -> Logos Storage
  -> CID
  -> LEZ registry entry
  -> query/resolve
  -> download
  -> byte verification
```

Everything else — CLI, SDK module, Basecamp UI, bulk workflows, update checks, adoption — should compose this same path rather than inventing parallel clients.

## Design principles

- **One core domain model.** CLI, Basecamp, and SDK should share region/registry semantics.
- **Ports around volatile infrastructure.** Geofabrik, Logos Storage, and LEZ should sit behind narrow interfaces so local tests can use fixtures without confusing them with real verification.
- **Content identity first.** CID/checksum/version/region-path relationships must remain explicit.
- **No mandatory hosted backend.** The allowed operational dependencies are the configured Logos stack and Geofabrik roles described by LP-0018.
- **Explicit environments.** Local fixture, standalone sequencer, and Logos testnet evidence must never be conflated.
- **Small vertical slices.** Prove one region end-to-end before adding bulk UI.

## Proposed repository shape

```text
/
├── AGENTS.md
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE-v2
├── docs/
│   ├── ARCHITECTURE.md
│   ├── ACCEPTANCE_MATRIX.md
│   ├── MISSIONS.md
│   ├── adr/
│   └── evidence/
├── crates/ or src/
│   ├── domain/                 # region + registry types, validation
│   ├── geofabrik/              # index, PBF, MD5/source metadata adapter
│   ├── storage/                # Logos Storage adapter
│   ├── registry/               # LEZ registry client/shared contract types
│   └── orchestration/          # host/download/import/update use cases
├── programs/
│   └── osm_registry/           # SPEL/LEZ on-chain program + IDL
├── modules/
│   ├── osm_registry_sdk/       # reusable Logos Basecamp/core module
│   └── osm_distribution_ui/    # Basecamp UI module/app
├── cli/
│   └── ...
└── tests/
    ├── fixtures/
    ├── integration/
    └── e2e/
```

The language/layout should follow what the current Logos module/LEZ toolchain makes easiest; do not lock a language merely to satisfy this sketch.

## Core domain objects

### Region

Minimum local representation:

```text
path            stable Geofabrik path / LP-0018 region id
name            display name
parent          parent path or null
level           country | subregion
source_url      canonical PBF URL
checksum_url    canonical published MD5 URL
version         snapshot/version representation
```

### VerifiedSnapshot

```text
region_path
source_url
local_path or byte source
expected_md5
observed_md5
byte_length
verified_at
```

This object should only exist after checksum comparison succeeds.

### StorageObject

```text
region_path
cid
byte_length
storage_endpoint/context
stored_at
```

### RegistryEntry

Must support at least the LP-0018 fields:

```text
region
parent
level
cid
source_url
checksum
version
hosted
timestamp
```

The region's Geofabrik path is the unique key. Queries must support region, parent, and CID.

## Ports / interfaces

### GeofabrikSource

Responsibilities:
- retrieve/parse index;
- restrict to frozen predefined region set;
- resolve region metadata;
- fetch checksum;
- fetch PBF;
- surface snapshot/version data.

### SnapshotVerifier

Responsibilities:
- stream/hash bytes;
- compare expected/observed MD5 for local import/source-verification flows;
- return structured mismatch errors.

### StoragePort

Responsibilities:
- store bytes/PBF;
- return CID;
- retrieve by CID;
- expose retryable vs terminal errors.

### RegistryPort

Responsibilities:
- register one region;
- batch-register regions;
- query by region;
- query by parent;
- query by CID;
- preserve timestamp/version semantics.

### DistributionService

Owns use cases rather than protocol details:
- discover;
- host;
- batch host;
- download hosted or fallback;
- local import;
- update check.

CLI, SDK and Basecamp call this layer or the same underlying APIs.

## First vertical slice (M06)

Use one predefined, relatively small region.

Required proof chain:

1. Resolve region from the published Geofabrik index.
2. Fetch its PBF and published checksum.
3. Verify downloaded bytes.
4. Store exact verified bytes in Logos Storage.
5. Obtain CID.
6. Register region metadata + CID through the LEZ registry.
7. Query the registry by region and recover the CID/metadata.
8. Fetch bytes from Logos Storage using the CID.
9. Demonstrate retrieved bytes correspond to the expected snapshot.

The same evidence packet should contain all IDs/URLs/hashes/commands required to reconstruct the chain.

## Environment model

### fixture
Fast deterministic tests. Never qualifies as Storage/LEZ verification.

### standalone
Real local/standalone Logos components, including real sequencer where required by the spec's E2E criterion. Label as local/standalone, not testnet.

### testnet-0.3
Actual Logos testnet 0.3 deployment and interactions. Only this environment may satisfy the prize's testnet claims.

## Integrity model

- Geofabrik is canonical for the source index and published MD5 in the roles allowed by the prize.
- Local import must fail closed on checksum mismatch.
- Stored hosted content is addressed by CID.
- Registry metadata must tie region path, CID, source/checksum and version together.
- Evidence should record both expected and observed digests when checksum verification is part of the flow.

## Reliability

Storage upload behavior should distinguish retryable errors and implement bounded exponential backoff. The user-facing layer must receive a terminal structured error after retries are exhausted.

Registry writes must make ordering/version semantics explicit; timestamps are not decorative metadata because the prize uses them for version sorting.

## SDK rule

The distribution app must use the same registry SDK/API surface that is documented for external consumers rather than a private parallel client. A limited consumer subset may expose region discovery/resolution without host/register operations, matching the prize specification.

## Non-goals for early milestones

Do not spend early cycles on:
- map rendering;
- tiles;
- routing;
- geocoding;
- custom region trees;
- large UI polish;
- analytics;
- a hosted relay/backend;
- adoption automation.

These either fall outside LP-0018 or are downstream of the technical proof.
