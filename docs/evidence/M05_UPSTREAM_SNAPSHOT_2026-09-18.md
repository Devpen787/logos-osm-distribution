# M05 LEZ/SPEL Upstream Snapshot — 2026-09-18

Mission: **M05 — minimal LEZ OSM registry proof**

## Pinned implementation baseline

### SPEL

- repository: `logos-co/spel`
- branch observed: `main`
- pinned commit: `512e95912a4e374686435601d6614b60a179a183`
- `spel-framework` package version at the pin: `0.7.0`

Current SPEL supports the vector argument shapes needed by the OSM batch API, including:

- `Vec<String>`
- `Vec<u64>`
- `Vec<bool>`

This means the OSM registry does not need the older comma-separated-string workaround used by earlier ecosystem code.

### Logos Execution Zone

Current SPEL scaffolding defaults to the stable LEZ tag:

- repository: `logos-blockchain/logos-execution-zone`
- tag: `v0.2.4`
- tag commit: `47eba256479f6f785acbd138834340703cd03401`

The current LEZ `dev` branch was also observed at:

`0fb0cb81f5adc2eed50494d98ef64530676dad2a`

M05 uses the stable `v0.2.4` dependency surface rather than chasing `dev`.

### Current official program examples

- repository: `logos-blockchain/lez-programs`
- main observed at: `f3dfe819004b29868d8f1c7f73cfdbccd967df3b`

The repository uses LEZ `v0.2.4`, generated/committed IDLs, and SPEL guest wrappers around separately testable Rust program logic.

## Prior accepted ecosystem reference

The accepted LP-0017 solution documented a LEZ SPEL registry with:

- a single registry PDA;
- shared off-chain/on-chain types;
- an `init_registry` instruction;
- batch registration;
- CID lookup;
- committed IDL;
- local and later testnet proof.

M05 uses that as architecture evidence only. OSM has different state/query requirements and the implementation is independently modeled.

## M05 truth boundary

M05 targets:

- deterministic registry logic in normal Rust tests;
- a compiling SPEL guest interface;
- generated IDL;
- a local/standalone LEZ execution proof.

M05 does **not** claim:

- Logos testnet deployment;
- Storage -> LEZ end-to-end host flow;
- final SDK/module integration;
- adoption evidence.

Those remain M11, M06, M08, and M14-M16 respectively.
