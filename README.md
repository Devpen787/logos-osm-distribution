# Logos OSM Distribution

Independent community implementation targeting **Logos λPrize LP-0018 — OpenStreetMap Integration: Decentralized Map Data Distribution**.

> This repository is not an official Logos project and is not endorsed, operated, or maintained by Logos. It is an independent implementation intended to satisfy the published LP-0018 specification.

## Status

**Bootstrap / pre-implementation.**

The current development environment has been locally verified on macOS Apple Silicon for the Logos module workflow: Nix, `logos-module-builder`, LGX packaging, `lgpm`, Basecamp, and `logoscore` all work end-to-end. That verification is environment evidence only; it is not evidence that this implementation satisfies LP-0018.

## Goal

Deliver a reproducible Logos-native distribution layer for verified OpenStreetMap PBF snapshots:

```text
Geofabrik
   |
   v
region discovery + published checksum
   |
   v
verified PBF snapshot
   |
   +--> Logos Storage --> CID
   |
   +--> LEZ OSM registry --> region metadata / CID / version
                            |
                            +--> SDK/module
                            +--> CLI
                            +--> Basecamp distribution app
```

The first technical proof target is intentionally smaller:

```text
one predefined region
  -> discover
  -> fetch PBF + checksum
  -> verify bytes
  -> store in Logos Storage
  -> register CID + metadata in LEZ
  -> query registry
  -> download by CID
  -> verify retrieved bytes
```

No broad UI or adoption work should begin until that vertical slice is proven.

## Required deliverables

The published LP-0018 specification requires, among other criteria:

- predefined Geofabrik region discovery;
- host workflow: Geofabrik -> Logos Storage -> LEZ registry;
- hosted download plus direct Geofabrik fallback;
- bulk hosting;
- local PBF import with Geofabrik MD5 verification;
- update checking;
- an on-chain OSM registry with region/parent/level/CID/source/checksum/version/hosted/timestamp fields;
- operation on Logos testnet 0.3;
- a reusable OSM registry SDK/Basecamp module;
- a Basecamp distribution application;
- a CLI;
- retries, integrity failures, timestamp ordering, and no mandatory centralized runtime service beyond the allowed Logos/Geofabrik roles;
- cycle-count documentation for LEZ operations;
- macOS Apple Silicon and Linux x86_64 support;
- module-catalog publishing;
- real-sequencer end-to-end CI;
- required adoption evidence.

See `docs/ACCEPTANCE_MATRIX.md` for the traceable implementation checklist.

## Repository boundaries

This repository contains the **actual Logos deliverable**.

It must remain independently buildable and evaluable. In particular:

- ProofForge is used externally for planning, evidence, and verification; it is not a runtime dependency.
- Palantir may be used externally as an operational/control-plane experiment; it is not a runtime dependency.
- TrailPassport may inform geospatial reasoning, but code is not copied into this project by default.

## Truth labels

Use precise evidence language:

- `source-confirmed`
- `implemented`
- `locally-verified`
- `testnet-verified`
- `externally-verified`
- `adoption-confirmed`
- `planned`
- `blocked`

A build passing locally is not testnet verification. A self-authored integration is not independent adoption. A mocked service is not a Logos Storage or LEZ proof.

## Development order

1. **M03 — Geofabrik source/checksum spike**
2. **M04 — Logos Storage spike**
3. **M05 — LEZ OSM registry spike**
4. **M06 — first complete vertical slice**
5. CLI + SDK/module
6. Basecamp app
7. reliability, CI, testnet, catalog
8. adoption and evaluator proof

See `docs/MISSIONS.md`.

## Licensing

This project is intended to be dual-licensed under:

- MIT (`LICENSE-MIT`)
- Apache License 2.0 (`LICENSE-APACHE-v2`)

Unless explicitly stated otherwise, contributions are submitted under both licenses.

## Upstream specification

Canonical prize source:

`logos-co/lambda-prize/prizes/LP-0018.md`

The specification is volatile external source material. Re-check it before material architecture changes, public eligibility claims, adoption campaigns, or a solution submission.
