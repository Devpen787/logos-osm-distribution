# Mission Queue

The queue is proof-oriented. Missions may be split further, but they should not be broadened until their primary proof target is clear.

## M02 — Evaluator-clean public repository

**Status:** complete

Goal:
- establish the independent public implementation repository;
- include README, dual licenses, AGENTS, architecture, acceptance matrix, mission queue;
- keep ProofForge and Palantir out of runtime dependencies.

Exit evidence:
- public repository URL;
- default branch contains bootstrap scaffold;
- clean clone is possible;
- license files present.

## M03 — Geofabrik discovery + checksum proof

**Status:** complete / locally verified

**Primary requirements:** F-01, R-02 foundation.

Goal:
- fetch/parse the current Geofabrik index;
- map it onto LP-0018's frozen predefined region set;
- resolve region path, parent/level, source URL, checksum URL, version metadata;
- download one small predefined region and verify its published MD5;
- prove checksum mismatch fails closed.

Do not include Logos Storage or LEZ yet.

Exit evidence:
- deterministic parser tests;
- frozen-set validation;
- one real region resolution;
- expected/observed MD5 evidence;
- negative checksum test.

## M04 — Logos Storage proof

**Primary requirements:** F-02/F-03 foundation, R-01 foundation.

Goal:
- take known verified bytes;
- store them through the current Logos Storage integration path;
- receive a CID;
- fetch the same object by CID;
- demonstrate byte identity/integrity;
- record transient/terminal error semantics for later retry work.

Do not register on LEZ yet.

Exit evidence:
- exact upstream Storage versions/refs;
- exact commands/config;
- returned CID;
- retrieval evidence;
- byte/hash comparison.

## M05 — Minimal LEZ OSM registry proof

**Primary requirements:** F-07, S-05; foundation for F-08/P-01.

Goal:
- implement the smallest SPEL/LEZ registry that supports the canonical `RegistryEntry` fields;
- register one entry;
- query by region;
- query by parent;
- query by CID;
- prove timestamp ordering semantics;
- define batch-register API even if bulk workflow comes later.

Start on a local/standalone environment. Do not claim testnet verification.

Exit evidence:
- source;
- generated IDL;
- local/standalone tests;
- register/query transcript;
- explicit environment label.

## M06 — First complete vertical slice

**Primary requirements:** F-02; foundation for F-03 and evaluator E2E.

Goal:

```text
Geofabrik
 -> resolve one predefined region
 -> fetch PBF + MD5
 -> verify bytes
 -> Logos Storage
 -> CID
 -> LEZ registry write
 -> registry query
 -> Storage retrieval by CID
 -> verify retrieved snapshot
```

This is the architecture gate. If it is awkward, refactor here before adding UI.

Exit evidence packet must tie together:
- region id/path;
- source URL;
- checksum URL/value;
- observed hash;
- Storage CID;
- registry transaction/write identity;
- queried registry entry;
- downloaded byte/hash identity;
- component commits and upstream refs.

## M07 — CLI + local import + download/fallback

**Primary requirements:** U-03, F-03, F-05.

Commands should cover at minimum:
- discover/list;
- host;
- batch-register/host support as architecture permits;
- lookup by region/parent/CID;
- download;
- import local PBF;
- update check.

## M08 — Registry SDK/Basecamp module

**Primary requirements:** F-09, U-02, S-03, S-08.

Goal:
- stable documented API;
- distribution app uses the same API;
- minimal consumer embedding example;
- dev + portable module builds kept explicit.

## M09 — Basecamp distribution app

**Primary requirements:** U-01, U-04, S-03.

Goal:
- region discovery and status;
- host/download/import/update user flows;
- clear verification state;
- install/load/use in Basecamp.

No map viewer. LP-0018 distributes snapshots; it does not render them.

## M10 — Bulk + reliability + update hardening

**Primary requirements:** F-04, F-06, R-01..R-04.

Goal:
- bulk host;
- bounded exponential backoff;
- clear terminal errors;
- timestamp/version behavior;
- dependency review proving no mandatory centralized backend.

## M11 — Logos testnet 0.3

**Primary requirements:** F-08, S-01, SUB-03.

Goal:
- deploy registry to canonical Logos testnet 0.3;
- run a bounded end-to-end proof;
- record program ID and transaction/query evidence.

Human review before consequential deployment/funding steps.

## M12 — CI + platform + performance evidence

**Primary requirements:** P-01, S-02, S-06.

Goal:
- real-sequencer standalone E2E in CI;
- green default branch;
- macOS Apple Silicon evidence;
- Linux x86_64 evidence;
- `cycle_bench` documentation/results for register + batch-register.

## M13 — Module catalog

**Primary requirements:** S-04, SUB-04.

Goal:
- publish distribution + SDK modules via the required module-catalog/release-action path;
- prove evaluator install path through `logos-repo.json`.

## M14 — Coverage adoption program

**Primary requirement:** A-01.

Goal:
- reach at least 15 countries and 25 verified region entries on the official zone/Storage;
- maintain evaluator-reproducible evidence for every counted entry.

Do not manufacture operators or entries merely to inflate counts.

## M15 — Independent SDK consumer program

**Primary requirement:** A-02.

Goal:
- 5 genuinely independent consuming modules;
- at least 3 Basecamp UI modules/apps;
- public code with real contributor history;
- inspectable use of the SDK/distribution module.

Projects controlled by the submitting team do not count.

## M16 — Discretionary ecosystem signals

**Primary requirements:** AD-01, AD-02.

Optional, only after required adoption is credible:
- redundant independent mirrors;
- genuine user/host vouching.

## M17 — Evaluator proof pack

Goal:
- clean-clone instructions;
- FURPS self-assessment;
- requirement/evidence index;
- program/catalog/adoption links;
- known limitations stated precisely;
- reproduce evaluator journey from scratch.

## M18 — λPrize solution submission

**Human-gated.**

Before opening:
- refresh prize status and terms;
- confirm eligibility;
- search for prior qualifying solution PRs;
- verify submission cadence/attempt limits;
- independently re-run evaluator path;
- obtain explicit human approval for the external solution PR.
