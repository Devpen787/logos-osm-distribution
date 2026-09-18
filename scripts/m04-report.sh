#!/usr/bin/env bash
set -u

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

WORK="$ROOT/.tmp/m04"
RUNTIME="$WORK/runtime"
TERMINAL="$WORK/m04-terminal.log"

echo "===== M04 QUICK REPORT ====="

echo
echo "=== GIT ==="
git branch --show-current 2>/dev/null || true
git rev-parse HEAD 2>/dev/null || true

echo
echo "=== LAST STAGE / KEY RESULTS ==="
if [ -f "$TERMINAL" ]; then
  grep -E     '^(===|M04 LOCAL STORAGE PROOF|BYTE-FOR-BYTE|CID:|Bytes:|MD5:|Timed out|logoscore exited|error:|Error:)'     "$TERMINAL" | tail -80 || true
else
  echo "No terminal log"
fi

echo
echo "=== CID ==="
cat "$RUNTIME/cid.txt" 2>/dev/null || echo "No CID file"

echo
echo "=== RETRIEVED VERIFICATION ==="
cat "$RUNTIME/retrieved-verification.json" 2>/dev/null || echo "No verification file"

echo
echo "=== MANIFEST ==="
cat "$RUNTIME/manifests.json" 2>/dev/null || echo "No manifest file"

echo
echo "=== LOGOSCORE STATUS SNAPSHOT ==="
cat "$RUNTIME/logoscore-status.json" 2>/dev/null || echo "No status snapshot"

echo
echo "=== LOGOSCORE LOG TAIL ==="
tail -80 "$RUNTIME/logoscore.log" 2>/dev/null || echo "No logoscore log"

echo
echo "=== STORAGE LOG TAIL ==="
tail -80 "$RUNTIME/storage-data/storage.log" 2>/dev/null || echo "No storage log"

echo
echo "=== TERMINAL TAIL ==="
tail -80 "$TERMINAL" 2>/dev/null || true
