# M04 Logos Storage Upstream Snapshot — 2026-09-18

Mission: **M04 — Logos Storage proof**

## Pinned upstream

Primary module:

- repository: `logos-co/logos-storage-module`
- branch observed: `master`
- commit: `bcc29f05e2a7b4a2d215d09640aaa7436359906b`
- observed upstream commit date: 2026-09-17
- module metadata version: `2.1.3`
- module name: `storage_module`
- type: `core`
- interface: `universal`

This revision is pinned for M04 so the evidence is reproducible even if upstream changes.

## Current official runtime path

The module's executable tutorial builds and drives Storage through:

1. `logos-co/logos-logoscore-cli`
2. `logos-co/logos-package-manager#cli`
3. `logos-co/logos-storage-module#lgx`
4. install the unsigned local LGX into a modules directory with `lgpm`
5. start `logoscore`
6. load `storage_module`
7. initialize/start the Storage node
8. upload a local file with `uploadUrl`
9. obtain its CID from Storage manifests / completion evidence
10. download by CID with `downloadToUrl`
11. verify round-trip bytes
12. stop/destroy Storage and stop the daemon

## API surface relevant to M04

Current `StorageModuleImpl` exposes:

- `init(config)`
- `start()`
- `uploadUrl(filePath, chunkSize)`
- `manifests()`
- `exists(cid)`
- `downloadToUrl(cid, filePath, local, chunkSize)`
- `downloadManifest(cid)`
- `stop()`
- `destroy()`

Important asynchronous events include:

- `storageStart`
- `storageUploadProgress`
- `storageUploadDone` — success payload includes CID
- `storageDownloadProgress`
- `storageDownloadDone`
- `storageDownloadManifestDone`

## M04 truth boundary

A successful local Storage node proof is **real Logos Storage code**, but it is not:

- Logos testnet verification;
- proof of remote replication;
- LEZ registration;
- evaluator coverage/adoption evidence.

M04 therefore targets the truth label `locally-verified`.

## Exact source bytes

M04 reuses the M03 Ethiopia snapshot:

- source: Geofabrik Ethiopia
- byte length: `139733363`
- expected/observed MD5: `426bd510159627dc139d4d0ad3bc6acd`

The Storage proof must preserve those exact bytes through:

`verified PBF -> Storage upload -> CID -> local Storage download by CID -> byte/hash comparison`.
