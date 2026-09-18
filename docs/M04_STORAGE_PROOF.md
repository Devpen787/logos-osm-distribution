# M04 — Real Logos Storage Proof

## Goal

Prove the current Logos Storage module can accept the already-verified Ethiopia PBF, return a CID, retrieve the content by that CID, and preserve the exact bytes.

This mission deliberately does **not** register anything in LEZ.

## Upstream pin

Use:

`logos-co/logos-storage-module@bcc29f05e2a7b4a2d215d09640aaa7436359906b`

Module metadata at that revision reports version `2.1.3`.

## Input artifact

Expected local M03 artifact:

`.tmp/m03/ethiopia-latest.osm.pbf`

Expected:

- bytes: `139733363`
- MD5: `426bd510159627dc139d4d0ad3bc6acd`

MD5 manifest:

`.tmp/m03/ethiopia-latest.osm.pbf.md5`

## Proof shape

```text
M03 verified Ethiopia PBF
  -> real storage_module
  -> uploadUrl
  -> Storage manifest/CID
  -> exists(cid) == true
  -> downloadToUrl(cid, local=true)
  -> retrieved PBF
  -> cmp original vs retrieved
  -> Geofabrik MD5 verification
```

## Why local=true first

The official Storage tutorial supports a real local Storage node and local CID round-trip. This proves module/libstorage integration and content identity without introducing peer discovery/network availability as a second variable.

Remote retrieval/replication remains later evidence and must not be inferred from this mission.

## Runtime workspace

Use:

`.tmp/m04/`

Do not commit generated LGX packages, Nix outputs, Storage databases, PBFs, logs, CIDs, or runtime config.

## Exit evidence

M04 is packet-ready only when all of the following are preserved:

- pinned Storage module commit/version;
- Storage LGX build success;
- installation + module load evidence;
- Storage node start evidence;
- input PBF byte length + M03 MD5;
- upload accepted;
- CID captured;
- `exists(cid)` confirms local content;
- download by CID completes;
- retrieved byte length;
- byte-for-byte comparison passes;
- retrieved MD5 matches Geofabrik's expected MD5;
- Storage/daemon clean shutdown;
- failures/gaps recorded exactly.

## Retry/error semantics observation

Record whether failures happen:

- synchronously at command dispatch;
- asynchronously through `storageUploadDone` / `storageDownloadDone`;
- because content is absent/local lookup fails;
- because Storage node start/init failed.

M04 observes these semantics. Bounded exponential retry implementation belongs to M10.
