#!/usr/bin/env bash
set -euo pipefail

# Validate SIS metrics and report JSON files against schemas.
# Usage:
#   scripts/validate-metrics.sh [--base <dir>]
# Defaults:
#   base dir = target/testing (workspace root)

BASE="target/testing"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --base)
      BASE="$2"; shift 2;;
    *) echo "Unknown arg: $1"; exit 2;;
  esac
done

METRICS_JSON="$BASE/metrics_dump.json"
REPORT_JSON="$BASE/validation_report.json"
METRICS_SCHEMA="docs/schemas/sis-metrics-v1.schema.json"
REPORT_SCHEMA="docs/schemas/validation-report-v1.schema.json"

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 not found" >&2; exit 1
fi

PY=python3

echo "[*] Ensuring jsonschema is available"
if ! "$PY" - <<'PY'
import sys
try:
    import jsonschema  # noqa: F401
    sys.exit(0)
except Exception:
    sys.exit(1)
PY
then
  echo "[i] Installing jsonschema into a temp venv"
  VENV=".venv-metrics-validate"
  if [[ ! -d "$VENV" ]]; then "$PY" -m venv "$VENV"; fi
  # shellcheck disable=SC1090
  source "$VENV/bin/activate"
  pip install -q jsonschema
  PY=python
fi

validate() {
  local json="$1" schema="$2"
  if [[ ! -f "$json" ]]; then
    echo "[!] Missing $json (skipping)"; return 0
  fi
  echo "[*] Validating $json against $schema"
  "$PY" - <<PY
import json,sys,jsonschema
schema=json.load(open('$schema'))
doc=json.load(open('$json'))
jsonschema.validate(doc, schema)
print('OK:', '$json')
PY
}

validate "$METRICS_JSON" "$METRICS_SCHEMA"
validate "$REPORT_JSON"  "$REPORT_SCHEMA"

echo "[*] Done"
