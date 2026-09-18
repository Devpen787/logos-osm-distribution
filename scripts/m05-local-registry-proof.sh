#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

WORK="$ROOT/.tmp/m05"
mkdir -p "$WORK"

IDL="$ROOT/artifacts/osm-registry.idl.json"
REAL_CID="zDvZRwzmDq9DKvNkN9JJhhN81TeQpcRPz3W1Aj4z1KHKqGmQL7vK"
REAL_CHECKSUM="426bd510159627dc139d4d0ad3bc6acd"
REAL_VERSION="1789685417"
REAL_SOURCE="https://download.geofabrik.de/africa/ethiopia-latest.osm.pbf"
SCAFFOLD_REV="fbde92521710209a171820c570ad8d955b399009"

if command -v lgs >/dev/null 2>&1; then
  LGS="$(command -v lgs)"
elif command -v logos-scaffold >/dev/null 2>&1; then
  LGS="$(command -v logos-scaffold)"
else
  cat >&2 <<EOF
logos-scaffold is not installed.

Install the pinned M05 scaffold revision once:

  cargo install --git https://github.com/logos-co/scaffold \
    --rev $SCAFFOLD_REV logos-scaffold

Then rerun this script.
EOF
  exit 2
fi

echo "=== M05 ENVIRONMENT ==="
"$LGS" --version
rustc --version
cargo --version

echo "=== M05 STATIC TESTS ==="
cargo test --workspace --all-targets
cargo test --manifest-path methods/guest/Cargo.toml
cargo run --quiet -p osm-registry-idl > "$WORK/generated-idl.json"
diff -u "$IDL" "$WORK/generated-idl.json"

echo "=== M05 SCAFFOLD RUN: SETUP -> BUILD -> LOCALNET -> TOPUP -> DEPLOY ==="
"$LGS" run --no-post-deploy 2>&1 | tee "$WORK/lgs-run.log"

echo "=== LOCATE GUEST BINARY ==="
GUEST_BIN=""
for search_root in "$ROOT/target/riscv-guest" "$ROOT/methods/target/riscv-guest"; do
  if [ -d "$search_root" ]; then
    GUEST_BIN="$(find "$search_root" -type f -path '*/release/osm_registry.bin' -print -quit 2>/dev/null || true)"
    if [ -n "$GUEST_BIN" ]; then
      break
    fi
  fi
done

if [ -z "$GUEST_BIN" ]; then
  echo "Could not locate release/osm_registry.bin" >&2
  "$LGS" report --out "$WORK/scaffold-report.tar.gz" --tail 150 >/dev/null 2>&1 || true
  exit 1
fi

echo "Guest binary: $GUEST_BIN"
printf '%s\n' "$GUEST_BIN" > "$WORK/guest-bin.txt"

export NSSA_WALLET_HOME_DIR="$ROOT/.scaffold/wallet"
export LEE_WALLET_HOME_DIR="$ROOT/.scaffold/wallet"
export NSSA_SEQUENCER_URL="http://127.0.0.1:3040"

echo "=== RESOLVE DEFAULT REGISTRAR ==="
REGISTRAR="$(sed -n 's/^default_address=//p' "$ROOT/.scaffold/state/wallet.state" 2>/dev/null | head -1)"
if [ -z "$REGISTRAR" ]; then
  "$LGS" wallet list --json > "$WORK/wallet-list.json"
  REGISTRAR="$(python3 - "$WORK/wallet-list.json" <<'PY'
import json, re, sys
obj=json.load(open(sys.argv[1]))
lines=obj.get("accounts", [])
preferred=[]
fallback=[]
for line in lines:
    m=re.search(r"(Public/[1-9A-HJ-NP-Za-km-z]+)", line)
    if not m:
        continue
    fallback.append(m.group(1))
    if line.lstrip().startswith("/ Public/"):
        preferred.append(m.group(1))
vals=preferred or fallback
print(vals[0] if vals else "")
PY
)"
fi

test -n "$REGISTRAR" || { echo "No public registrar account found" >&2; exit 1; }
echo "Registrar: $REGISTRAR"
printf '%s\n' "$REGISTRAR" > "$WORK/registrar.txt"

echo "=== DERIVE REGISTRY PDA ==="
PDA="$("$LGS" spel -- -i "$IDL" -p "$GUEST_BIN" pda registry | tail -1)"
if [[ "$PDA" != Public/* ]]; then
  PDA="Public/$PDA"
fi
echo "Registry PDA: $PDA"
printf '%s\n' "$PDA" > "$WORK/registry-pda.txt"

fetch_state() {
  local label="$1"
  local wallet_out="$WORK/$label-account.txt"
  local state_bin="$WORK/$label-state.bin"

  "$LGS" wallet -- account get --account-id "$PDA" > "$wallet_out" 2>&1

  python3 - "$wallet_out" "$state_bin" <<'PY'
import base64, re, sys
text=open(sys.argv[1]).read()
m=re.search(r'"data_b64"\s*:\s*"([^"]*)"', text)
if not m:
    print(text, file=sys.stderr)
    raise SystemExit("account output did not contain data_b64")
open(sys.argv[2], "wb").write(base64.b64decode(m.group(1)))
PY

  echo "$state_bin"
}

query_state() {
  local state="$1"
  shift
  cargo run --quiet -p osm-registry-core --bin registry-state -- "$state" "$@"
}

echo "=== INITIALIZE REGISTRY ==="
"$LGS" spel --   -i "$IDL"   -p "$GUEST_BIN"   -- init-registry   --registrar "$REGISTRAR"   2>&1 | tee "$WORK/init-registry.log"

sleep 1
INIT_STATE="$(fetch_state init)"
query_state "$INIT_STATE" decode | tee "$WORK/init-decoded.json"

python3 - "$WORK/init-decoded.json" <<'PY'
import json, sys
obj=json.load(open(sys.argv[1]))
assert obj["schema_version"] == 1
assert obj["entries"] == []
print("INIT REGISTRY STATE: PASS")
PY

T1="$(date +%s)"
T2="$((T1 + 20))"
T3="$((T1 + 30))"

echo "=== REGISTER REAL M04 ETHIOPIA CID ==="
"$LGS" spel --   -i "$IDL"   -p "$GUEST_BIN"   -- register-batch   --registrar "$REGISTRAR"   --regions ethiopia   --parents ""   --levels 0   --cids "$REAL_CID"   --source-urls "$REAL_SOURCE"   --checksums "$REAL_CHECKSUM"   --versions "$REAL_VERSION"   --hosted true   --timestamps "$T1"   2>&1 | tee "$WORK/register-ethiopia.log"

sleep 1
ETH_STATE="$(fetch_state ethiopia)"
query_state "$ETH_STATE" decode | tee "$WORK/ethiopia-decoded.json"
query_state "$ETH_STATE" region ethiopia | tee "$WORK/query-region-ethiopia.json"
query_state "$ETH_STATE" cid "$REAL_CID" | tee "$WORK/query-real-cid.json"

python3 - "$WORK/ethiopia-decoded.json" "$REAL_CID" "$REAL_CHECKSUM" "$REAL_VERSION" <<'PY'
import json, sys
obj=json.load(open(sys.argv[1]))
assert len(obj["entries"]) == 1
e=obj["entries"][0]
assert e["region"] == "ethiopia"
assert e["parent"] is None
assert e["cid"] == sys.argv[2]
assert e["checksum"] == sys.argv[3]
assert e["version"] == int(sys.argv[4])
assert e["hosted"] is True
print("REAL ETHIOPIA REGISTRY ENTRY: PASS")
PY

echo "=== IDEMPOTENT REPLAY OF SAME SNAPSHOT ==="
"$LGS" spel --   -i "$IDL"   -p "$GUEST_BIN"   -- register-batch   --registrar "$REGISTRAR"   --regions ethiopia   --parents ""   --levels 0   --cids "$REAL_CID"   --source-urls "$REAL_SOURCE"   --checksums "$REAL_CHECKSUM"   --versions "$REAL_VERSION"   --hosted true   --timestamps "$((T1 + 1))"   2>&1 | tee "$WORK/replay-ethiopia.log"

sleep 1
REPLAY_STATE="$(fetch_state replay)"
cmp "$ETH_STATE" "$REPLAY_STATE"
echo "IDEMPOTENT REPLAY STATE IDENTITY: PASS"

echo "=== REGISTER SYNTHETIC LOCAL PARENT-QUERY FIXTURES ==="
"$LGS" spel --   -i "$IDL"   -p "$GUEST_BIN"   -- register-batch   --registrar "$REGISTRAR"   --regions us/california   --regions us/texas   --parents us   --parents us   --levels 1,1   --cids local-m05-unhosted-california   --cids local-m05-unhosted-texas   --source-urls https://download.geofabrik.de/north-america/us/california-latest.osm.pbf   --source-urls https://download.geofabrik.de/north-america/us/texas-latest.osm.pbf   --checksums 00000000000000000000000000000000   --checksums 11111111111111111111111111111111   --versions 1,1   --hosted false,false   --timestamps "$T2,$T3"   2>&1 | tee "$WORK/register-parent-fixtures.log"

sleep 1
FINAL_STATE="$(fetch_state final)"
query_state "$FINAL_STATE" decode | tee "$WORK/final-decoded.json"
query_state "$FINAL_STATE" parent us | tee "$WORK/query-parent-us.json"
query_state "$FINAL_STATE" region ethiopia | tee "$WORK/query-region-final.json"
query_state "$FINAL_STATE" cid "$REAL_CID" | tee "$WORK/query-cid-final.json"

python3 - "$WORK/final-decoded.json" "$WORK/query-parent-us.json" "$REAL_CID" <<'PY'
import json, sys
registry=json.load(open(sys.argv[1]))
us=json.load(open(sys.argv[2]))
assert len(registry["entries"]) == 3
assert [e["region"] for e in registry["entries"][:2]] == ["us/texas", "us/california"]
assert [e["region"] for e in us] == ["us/texas", "us/california"]
eth=[e for e in registry["entries"] if e["region"]=="ethiopia"]
assert len(eth)==1 and eth[0]["cid"]==sys.argv[3] and eth[0]["hosted"] is True
print("REGION/PARENT/CID + TIMESTAMP ORDERING: PASS")
PY

echo "=== STALE WRITE FAIL-CLOSED PROBE ==="
set +e
"$LGS" spel --   -i "$IDL"   -p "$GUEST_BIN"   -- register-batch   --registrar "$REGISTRAR"   --regions ethiopia   --parents ""   --levels 0   --cids local-m05-stale-ethiopia   --source-urls "$REAL_SOURCE"   --checksums "$REAL_CHECKSUM"   --versions "$REAL_VERSION"   --hosted false   --timestamps "$T1"   > "$WORK/stale-write.log" 2>&1
STALE_RC=$?
set -e

sleep 1
AFTER_STALE="$(fetch_state after-stale)"
cmp "$FINAL_STATE" "$AFTER_STALE"
echo "STALE WRITE LEFT REGISTRY STATE UNCHANGED: PASS"
echo "Stale-write CLI exit code: $STALE_RC"

echo "=== FINAL REGISTRY ==="
query_state "$AFTER_STALE" decode | tee "$WORK/final-registry.json"

"$LGS" localnet status --json > "$WORK/localnet-status.json" 2>&1 || true
"$LGS" report --out "$WORK/scaffold-report.tar.gz" --tail 150 >/dev/null 2>&1 || true

echo
echo "M05 LOCAL LEZ REGISTRY PROOF: PASS"
echo "Registry PDA: $PDA"
echo "Registrar: $REGISTRAR"
echo "Real Ethiopia CID: $REAL_CID"
echo "Entries: 3 (1 real M04-backed + 2 explicitly synthetic local query fixtures)"
echo "Testnet claim: NONE"
