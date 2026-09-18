#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

WORK="$ROOT/.tmp/m04"
RUNTIME="$WORK/runtime"
PBF="$ROOT/.tmp/m03/ethiopia-latest.osm.pbf"
MD5_MANIFEST="$ROOT/.tmp/m03/ethiopia-latest.osm.pbf.md5"
EXPECTED_SIZE=139733363
EXPECTED_MD5=426bd510159627dc139d4d0ad3bc6acd
STORAGE_REV=bcc29f05e2a7b4a2d215d09640aaa7436359906b

LOGOS="$RUNTIME/logos"
LGPM="$RUNTIME/lgpm"
STORAGE_LGX="$RUNTIME/storage-lgx"
MODULES="$RUNTIME/modules"
LOGOSCORE_CFG="$RUNTIME/logoscore-config"
STORAGE_DATA="$RUNTIME/storage-data"
CONFIG="$RUNTIME/storage-config.json"
LOGOSCORE_LOG="$RUNTIME/logoscore.log"
UPLOAD_CALL="$RUNTIME/upload-call.json"
MANIFESTS_JSON="$RUNTIME/manifests.json"
CID_FILE="$RUNTIME/cid.txt"
DOWNLOADED="$RUNTIME/ethiopia-retrieved.osm.pbf"
VERIFY_JSON="$RUNTIME/retrieved-verification.json"

cleanup() {
  set +e
  if [ -x "$LOGOS/bin/logoscore" ]; then
    "$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" call storage_module stop >/dev/null 2>&1
    sleep 1
    "$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" call storage_module destroy >/dev/null 2>&1
    "$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" stop >/dev/null 2>&1
  fi
}
trap cleanup EXIT

echo "=== M04 PRECHECK ==="
test -f "$PBF" || { echo "Missing $PBF"; exit 1; }
test -f "$MD5_MANIFEST" || { echo "Missing $MD5_MANIFEST"; exit 1; }

ACTUAL_SIZE="$(python3 -c 'import os,sys; print(os.path.getsize(sys.argv[1]))' "$PBF")"
echo "Input bytes: $ACTUAL_SIZE"
test "$ACTUAL_SIZE" = "$EXPECTED_SIZE"

echo "=== REVERIFY M03 INPUT ==="
cargo run -p geofabrik --bin geofabrik-check -- verify "$PBF" "$MD5_MANIFEST"

mkdir -p "$RUNTIME"
rm -rf "$MODULES" "$LOGOSCORE_CFG" "$STORAGE_DATA"
rm -f "$LOGOS" "$LGPM" "$STORAGE_LGX" "$DOWNLOADED"
mkdir -p "$MODULES" "$LOGOSCORE_CFG" "$STORAGE_DATA"

echo "=== BUILD LOGOSCORE ==="
nix build 'github:logos-co/logos-logoscore-cli' --out-link "$LOGOS"

echo "=== BUILD LGPM ==="
nix build 'github:logos-co/logos-package-manager#cli' --out-link "$LGPM"

echo "=== BUILD PINNED STORAGE MODULE ==="
nix build "github:logos-co/logos-storage-module/$STORAGE_REV#lgx" --out-link "$STORAGE_LGX"

echo "=== INSTALL STORAGE MODULE ==="
"$LGPM/bin/lgpm" \
  --modules-dir "$MODULES" \
  --allow-unsigned \
  install --file "$STORAGE_LGX"/*.lgx

"$LGPM/bin/lgpm" --modules-dir "$MODULES" list

cat > "$CONFIG" <<EOF
{
  "data-dir": "$STORAGE_DATA",
  "log-level": "DEBUG",
  "log-file": "$STORAGE_DATA/storage.log",
  "nat": "extip:127.0.0.1",
  "no-bootstrap-node": true,
  "mix-enabled": false
}
EOF

echo "=== START LOGOSCORE ==="
"$LOGOS/bin/logoscore" \
  --config-dir "$LOGOSCORE_CFG" \
  -D -m "$MODULES" \
  > "$LOGOSCORE_LOG" 2>&1 &
DAEMON_PID=$!
sleep 3
ps -p "$DAEMON_PID" >/dev/null

"$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" status
"$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" list-modules

echo "=== LOAD STORAGE MODULE ==="
"$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" load-module storage_module
"$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" module-info storage_module

echo "=== INIT STORAGE NODE ==="
"$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" \
  call storage_module init "@$CONFIG"

echo "=== START STORAGE NODE ==="
"$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" \
  call storage_module start
sleep 5

echo "=== STORAGE DEBUG ==="
"$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" \
  call storage_module debug

echo "=== UPLOAD VERIFIED ETHIOPIA PBF ==="
"$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" \
  call storage_module uploadUrl "$PBF" 65536 \
  | tee "$UPLOAD_CALL"

echo "=== WAIT FOR CID ==="
CID=""
for _ in $(seq 1 300); do
  if "$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" \
      call storage_module manifests > "$MANIFESTS_JSON" 2>/dev/null; then
    CID="$(python3 - "$MANIFESTS_JSON" "$EXPECTED_SIZE" <<'PY'
import json, sys
path, expected = sys.argv[1], int(sys.argv[2])
try:
    data = json.load(open(path))
    items = data.get("result", {}).get("value", [])
    for item in items:
        if int(item.get("datasetSize", -1)) == expected:
            print(item.get("cid", ""))
            break
except Exception:
    pass
PY
)"
  fi
  if [ -n "$CID" ]; then
    break
  fi
  sleep 2
done

test -n "$CID" || {
  echo "Timed out waiting for uploaded CID"
  tail -200 "$LOGOSCORE_LOG" || true
  exit 1
}
printf '%s\n' "$CID" | tee "$CID_FILE"
echo "CID: $CID"

echo "=== EXISTS LOCALLY ==="
"$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" \
  call storage_module exists "$CID"

echo "=== DOWNLOAD BY CID (LOCAL STORAGE ONLY) ==="
rm -f "$DOWNLOADED"
"$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" \
  call storage_module downloadToUrl "$CID" "$DOWNLOADED" true 65536

echo "=== WAIT FOR EXACT ROUND-TRIP ==="
ROUNDTRIP_OK=0
for _ in $(seq 1 300); do
  if [ -f "$DOWNLOADED" ]; then
    SIZE="$(python3 -c 'import os,sys; print(os.path.getsize(sys.argv[1]))' "$DOWNLOADED" 2>/dev/null || echo 0)"
    if [ "$SIZE" = "$EXPECTED_SIZE" ] && cmp -s "$PBF" "$DOWNLOADED"; then
      ROUNDTRIP_OK=1
      break
    fi
  fi
  sleep 2
done

test "$ROUNDTRIP_OK" = "1" || {
  echo "Timed out waiting for byte-identical Storage download"
  tail -200 "$LOGOSCORE_LOG" || true
  exit 1
}

echo "BYTE-FOR-BYTE ROUND-TRIP: PASS"

echo "=== VERIFY RETRIEVED BYTES AGAINST GEOFABRIK MD5 ==="
cargo run -p geofabrik --bin geofabrik-check -- \
  verify "$DOWNLOADED" "$MD5_MANIFEST" \
  | tee "$VERIFY_JSON"

OBSERVED="$(python3 - "$VERIFY_JSON" <<'PY'
import json, sys
print(json.load(open(sys.argv[1]))["observed_md5"])
PY
)"
test "$OBSERVED" = "$EXPECTED_MD5"

echo "=== STORAGE MANIFEST ==="
cat "$MANIFESTS_JSON"

echo "=== STORAGE LOG TAIL ==="
tail -100 "$STORAGE_DATA/storage.log" || true

echo "=== LOGOSCORE LOG TAIL ==="
tail -100 "$LOGOSCORE_LOG" || true

echo "=== CLEAN SHUTDOWN ==="
"$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" call storage_module stop
sleep 2
"$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" call storage_module destroy
"$LOGOS/bin/logoscore" --config-dir "$LOGOSCORE_CFG" stop
trap - EXIT

echo
echo "M04 LOCAL STORAGE PROOF: PASS"
echo "CID: $CID"
echo "Bytes: $EXPECTED_SIZE"
echo "MD5: $EXPECTED_MD5"
