#!/usr/bin/env bash
set -euo pipefail

# Quick extractor for key metrics from target/testing/metrics_dump.json
# Usage:
#   scripts/extract-metrics.sh [metrics_dump.json]
# Default path: target/testing/metrics_dump.json

FILE="${1:-target/testing/metrics_dump.json}"

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required (brew install jq / apt-get install jq)" >&2
  exit 1
fi

if [[ ! -f "$FILE" ]]; then
  echo "error: metrics file not found: $FILE" >&2
  exit 2
fi

echo "Metrics file: $FILE"

ctx_p95=$(jq -r '.summary.context_switch_p95_ns // empty' "$FILE")
ai_p99=$(jq -r '.summary.ai_inference_p99_us // empty' "$FILE")
mem_p99=$(jq -r '.summary.memory_allocation_p99_ns // empty' "$FILE")

echo "Context P95 (ns):      ${ctx_p95:-N/A}"
echo "AI Inference P99 (µs): ${ai_p99:-N/A}"
echo "Alloc P99 (ns):        ${mem_p99:-N/A}"

samples_real=$(jq -r 'if has("real_ctx_switch_ns") then (.real_ctx_switch_ns | length) else 0 end' "$FILE")
echo "real_ctx_switch_ns samples: $samples_real"

if [[ "${samples_real}" != "0" ]]; then
  jq -r '
    def pct($a; $p): ($a|sort) as $s | $s[(($s|length - 1) * $p)|floor];
    "real_ctx_switch_ns P50=\(pct(.real_ctx_switch_ns; 0.50)) ns, " +
    "P95=\(pct(.real_ctx_switch_ns; 0.95)) ns, " +
    "P99=\(pct(.real_ctx_switch_ns; 0.99)) ns"' "$FILE"
fi

