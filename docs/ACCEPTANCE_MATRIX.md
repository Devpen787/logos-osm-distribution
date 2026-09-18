# LP-0018 Acceptance Matrix

This file maps the published LP-0018 outcome criteria to implementation missions and evidence. It is not a substitute for the canonical prize specification.

Status vocabulary:

- `unaddressed`
- `in_progress`
- `implemented`
- `locally_verified`
- `testnet_verified`
- `externally_verified`
- `adoption_confirmed`
- `blocked`

## Functionality

| ID | Criterion | Primary mission | Minimum evidence | Status |
|---|---|---|---|---|
| F-01 | Discover predefined Geofabrik regions with version + level | M03 | parsed canonical index; frozen-set filtering tests | in_progress |
| F-02 | Host: Geofabrik -> Logos Storage -> LEZ registry | M06 | end-to-end command/log + CID + registry query | unaddressed |
| F-03 | Download from Logos Storage when hosted, Geofabrik fallback otherwise | M06/M07 | hosted and fallback integration tests | unaddressed |
| F-04 | Bulk host with per-region opt-out and overlap safety if extras exposed | M10 | multi-region test + UX/CLI evidence | unaddressed |
| F-05 | Local PBF import verifies Geofabrik MD5 then stores/registers | M07 | valid + mismatch fixture and real integration evidence | unaddressed |
| F-06 | Update check compares source versions vs registry by region path | M10 | stale/current cases | unaddressed |
| F-07 | LEZ registry stores required fields and queries region/parent/CID; supports batch register | M05 | program tests + IDL + query/batch evidence | unaddressed |
| F-08 | Registry operates against Logos testnet 0.3 | M11 | program ID + testnet transaction/query evidence | unaddressed |
| F-09 | Self-contained OSM registry SDK/Basecamp module with stable API + embedding example | M08 | published API docs + independent example module build | unaddressed |

## Usability

| ID | Criterion | Primary mission | Minimum evidence | Status |
|---|---|---|---|---|
| U-01 | Basecamp distribution app GUI, build assets/instructions, loadable in Basecamp | M09 | clean build/install/use evidence | unaddressed |
| U-02 | SDK/module usable by other Basecamp modules/apps, docs + embedding example | M08 | documented API + example | unaddressed |
| U-03 | CLI covers host, batch-register, lookups, update, download, local import | M07 | command-level tests + docs | unaddressed |
| U-04 | Clear UX for hosted state, versions, verification results | M09 | UI test evidence/screens | unaddressed |

## Reliability

| ID | Criterion | Primary mission | Minimum evidence | Status |
|---|---|---|---|---|
| R-01 | Storage retries transient upload failures with bounded exponential backoff | M10 | deterministic failure/retry tests | unaddressed |
| R-02 | Checksum mismatch fails gracefully with clear error | M03/M07 | mismatch test + user-facing error evidence | locally_verified |
| R-03 | Registry timestamps support version ordering | M05 | ordering tests | unaddressed |
| R-04 | No mandatory centralized service beyond allowed Logos + Geofabrik roles | all | architecture/dependency review + clean E2E | unaddressed |

## Performance

| ID | Criterion | Primary mission | Minimum evidence | Status |
|---|---|---|---|---|
| P-01 | Document LEZ `cycle_bench` counts for register + batch-register | M12 | reproducible benchmark commands/results | unaddressed |

## Supportability

| ID | Criterion | Primary mission | Minimum evidence | Status |
|---|---|---|---|---|
| S-01 | Deploy/test on Logos testnet 0.3 | M11 | program ID + testnet E2E | unaddressed |
| S-02 | macOS Apple Silicon + Linux x86_64 end-to-end | M12 | clean-run evidence on both architectures | unaddressed |
| S-03 | Basecamp modules use `logos-module-builder` / `mkLogosModule` | M08/M09 | flake/module build evidence | unaddressed |
| S-04 | Publish modules to module catalog via required release flow | M13 | catalog URL + install evidence | unaddressed |
| S-05 | Registry IDL generated via SPEL | M05 | source + generated IDL | unaddressed |
| S-06 | Real-sequencer standalone host->store->register->query->download E2E in green CI | M12 | CI workflow + run logs | unaddressed |
| S-07 | README covers setup, program ID, region list, CLI + Basecamp usage | M17 | evaluator-ready README | unaddressed |
| S-08 | SDK docs include worked region->CID/metadata resolution | M08 | docs + executable example | unaddressed |

## Required adoption

These criteria are not satisfied by software produced or controlled by the submitting team.

| ID | Criterion | Primary mission | Minimum evidence | Status |
|---|---|---|---|---|
| A-01 | >=15 countries + >=25 verified region entries on official Logos testnet 0.3 / Storage | M14 | region/CID/registry dataset independently reproducible by evaluator | unaddressed |
| A-02 | >=5 independent consuming modules, >=3 Basecamp UI apps; genuine public history; one may use standalone SDK only | M15 | repository links + code inspection + independence evidence | unaddressed |

## Discretionary adoption

| ID | Signal | Primary mission | Evidence | Status |
|---|---|---|---|---|
| AD-01 | Redundant mirroring by >5 distinct operators | M16 | same CID hosted by distinct genuine operators | unaddressed |
| AD-02 | Genuine community vouching from users/hosts | M16 | attributable public evidence with real account history | unaddressed |

## Submission

| ID | Requirement | Primary mission | Minimum evidence | Status |
|---|---|---|---|---|
| SUB-01 | Public original-work repository | M02 | public repository | satisfied |
| SUB-02 | Dual MIT + Apache-2.0 licensing | M02 | both license files + README declaration | satisfied |
| SUB-03 | Registry IDL + testnet 0.3 program ID | M11/M17 | source/IDL/deployment record | unaddressed |
| SUB-04 | Module catalog + `logos-repo.json` URL | M13 | public catalog/install evidence | unaddressed |
| SUB-05 | Required adoption evidence | M14/M15 | coverage + consumer links | unaddressed |
| SUB-06 | FURPS self-assessment | M17 | completed assessment mapped to evidence | unaddressed |
| SUB-07 | λPrize solution PR | M18 | human-approved PR | human_gated |

## Eligibility gate

Current λPrize policy excludes Logos service providers and their contractors/subcontractors ("Logos CCs") from participation and from counting toward adoption criteria.

Maintainer self-attestation recorded for this project: **not a Logos CC**. This is a project-control record, not legal advice; re-check eligibility before submission if circumstances change.

## Current critical path

```text
M02 public repo
  |
  +--> M03 Geofabrik source/checksum
  +--> M04 Logos Storage
  +--> M05 LEZ registry
          |
          v
       M06 first vertical slice
          |
          +--> M07 CLI/import/download
          +--> M08 SDK module
          +--> M09 Basecamp app
          +--> M10 reliability/bulk/update
          +--> M11 testnet
          +--> M12 CI/platform/cycle evidence
          +--> M13 catalog
          +--> M14/M15 adoption
          +--> M17 evaluator pack
          +--> M18 human-gated solution PR
```
