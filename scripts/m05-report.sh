#!/usr/bin/env bash
set -u

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"
WORK="$ROOT/.tmp/m05"

echo "===== M05 QUICK REPORT ====="

echo
echo "=== GIT ==="
git branch --show-current 2>/dev/null || true
git rev-parse HEAD 2>/dev/null || true

echo
echo "=== KEY RESULTS ==="
grep -Eh   '^(===|M05 LOCAL LEZ REGISTRY PROOF|Registry PDA:|Registrar:|Real Ethiopia CID:|Entries:|Testnet claim:|.*PASS$|Stale-write CLI exit code:)'   "$WORK"/*.log "$WORK"/*.txt 2>/dev/null | tail -120 || true

echo
echo "=== REGISTRY PDA ==="
cat "$WORK/registry-pda.txt" 2>/dev/null || echo "No PDA file"

echo
echo "=== FINAL REGISTRY ==="
cat "$WORK/final-registry.json" 2>/dev/null || echo "No final registry"

echo
echo "=== REAL CID QUERY ==="
cat "$WORK/query-cid-final.json" 2>/dev/null || echo "No CID query"

echo
echo "=== PARENT QUERY ==="
cat "$WORK/query-parent-us.json" 2>/dev/null || echo "No parent query"

echo
echo "=== STALE WRITE ==="
tail -80 "$WORK/stale-write.log" 2>/dev/null || echo "No stale-write log"

echo
echo "=== LOCALNET STATUS ==="
cat "$WORK/localnet-status.json" 2>/dev/null || echo "No localnet status"

echo
echo "=== LGS RUN TAIL ==="
tail -100 "$WORK/lgs-run.log" 2>/dev/null || echo "No lgs-run log"
