# ADR 0001 — Rust for the shared core

Status: proposed for M03 implementation

## Context

LP-0018 spans source ingestion, hashing, CLI work, Logos modules, and a SPEL/LEZ registry. We want one core domain model rather than separate implementations per surface.

Current upstream fit supports Rust in both important directions:

- `logos-module-builder` provides a first-class `#rust` core-module template and Rust module backend.
- SPEL/LEZ program and client tooling is Rust/Cargo-native and generates typed Rust clients/IDL artifacts.

## Decision

Use Rust for the shared domain and infrastructure crates unless a later Logos integration proves a narrower language boundary is necessary.

M03 starts with two crates:

- `osm-domain`: frozen LP-0018 region identities and normalized domain types.
- `geofabrik`: Geofabrik index parsing and streaming checksum verification.

The decision does not force the Basecamp UI itself to be Rust. QML remains the expected UI layer unless current Logos tooling suggests otherwise when M09 starts.

## Consequences

Positive:
- shared types can flow toward CLI, registry client, and Rust Logos modules;
- streaming hashing is straightforward and memory-safe for large PBF files;
- SPEL/LEZ integration avoids a cross-language core rewrite;
- unit tests can remain deterministic and independent of network services.

Tradeoffs:
- QML/UI integration will still cross a module boundary;
- current Logos Rust module behavior must be pinned and re-verified when M08 begins;
- HTTP transport is intentionally not selected in M03; parsing/verification stays transport-independent first.

## Revisit trigger

Revisit only if the real M04/M05/M06 vertical slice demonstrates that the chosen boundary creates duplicated logic or an unsupported Logos runtime path.
