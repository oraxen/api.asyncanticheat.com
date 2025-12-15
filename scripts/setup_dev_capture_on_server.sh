#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   setup_dev_capture_on_server.sh <player_name> <player_uuid>
#
# What it does:
# - Ensures AsyncAnticheat plugin points at the local API with the correct ingest token
# - Ensures dev mode is enabled (does not change it if already set)
# - Adds the player to ops.json (so /aacdev works without a permissions plugin)
# - Restarts paper-test once (so op + config apply)

PLAYER_NAME="${1:-}"
PLAYER_UUID="${2:-}"

if [[ -z "$PLAYER_NAME" || -z "$PLAYER_UUID" ]]; then
  echo "Usage: $0 <player_name> <player_uuid>" >&2
  exit 2
fi

PAPER_DIR="/root/minecraft/paper-1.21"
AAC_CFG="${PAPER_DIR}/plugins/AsyncAnticheat/config.yml"
OPS_JSON="${PAPER_DIR}/ops.json"
API_ENV="/opt/async_anticheat_api/.env"

if [[ ! -f "$AAC_CFG" ]]; then
  echo "ERROR: missing AsyncAnticheat config: $AAC_CFG" >&2
  exit 1
fi
if [[ ! -f "$API_ENV" ]]; then
  echo "ERROR: missing API env file: $API_ENV" >&2
  exit 1
fi

INGEST_TOKEN="$(grep -E '^INGEST_TOKEN=' "$API_ENV" | head -1 | cut -d= -f2- || true)"
if [[ -z "$INGEST_TOKEN" ]]; then
  echo "ERROR: INGEST_TOKEN not found in $API_ENV" >&2
  exit 1
fi

echo "[1/3] Updating AsyncAnticheat api.url/api.token (token not printed)..."
python3 - <<PY
import yaml

cfg_path = "${AAC_CFG}"
ingest_token = "${INGEST_TOKEN}"

with open(cfg_path, "r") as f:
    data = yaml.safe_load(f) or {}

api = data.get("api") or {}
api["url"] = "http://localhost:3002"
api["token"] = ingest_token
api.setdefault("timeout_seconds", 10)
data["api"] = api

dev = data.get("dev") or {}
dev.setdefault("enabled", True)
dev.setdefault("default_duration_seconds", 60)
dev.setdefault("default_warmup_seconds", 3)
dev.setdefault("default_toggle_seconds", 10)
data["dev"] = dev

with open(cfg_path, "w") as f:
    yaml.safe_dump(data, f, sort_keys=False)

print("OK:")
print("  api.url =", api.get("url"))
print("  api.token_len =", len(api.get("token", "")))
print("  dev.enabled =", dev.get("enabled"))
PY

echo "[2/3] Ensuring OP via ops.json..."
python3 - <<PY
import json
from pathlib import Path

ops_path = Path("${OPS_JSON}")
name = "${PLAYER_NAME}"
uuid = "${PLAYER_UUID}"

entry = {
    "uuid": uuid,
    "name": name,
    "level": 4,
    "bypassesPlayerLimit": False,
}

ops = []
if ops_path.exists():
    try:
        ops = json.loads(ops_path.read_text("utf-8") or "[]")
        if not isinstance(ops, list):
            ops = []
    except Exception:
        ops = []

ops = [o for o in ops if not (isinstance(o, dict) and (o.get("uuid") == uuid or o.get("name") == name))]
ops.append(entry)
ops_path.write_text(json.dumps(ops, indent=2) + "\n", "utf-8")
print("OK wrote ops.json entry for", name, uuid)
PY

echo "[3/3] Restarting paper-test to apply config + op..."
systemctl restart paper-test
sleep 3

echo "paper-test status:"
systemctl status paper-test --no-pager | head -15

echo
echo "Recent AsyncAnticheat upload lines:"
journalctl -u paper-test -n 200 --no-pager | grep -E "AsyncAnticheat.*Upload|Entering spool-only|Started\\.|session_id=" | tail -30 || true


