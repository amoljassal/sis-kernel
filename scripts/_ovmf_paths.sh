#!/usr/bin/env bash
set -euo pipefail

# Resolve OVMF paths. Caller may override via env:
#   OVMF_CODE / OVMF_VARS

if [[ -n "${OVMF_CODE:-}" && -n "${OVMF_VARS:-}" ]]; then
  exit 0
fi

detect_mac() {
  local base="/opt/homebrew/opt/edk2-ovmf/share/edk2-ovmf/x64"
  [[ -f "$base/OVMF_CODE.fd" && -f "$base/OVMF_VARS.fd" ]] || return 1
  OVMF_CODE="$base/OVMF_CODE.fd"
  OVMF_VARS="$base/OVMF_VARS.fd"
}

detect_linux() {
  local cands=(
    "/usr/share/OVMF/OVMF_CODE.fd:/usr/share/OVMF/OVMF_VARS.fd"
    "/usr/share/OVMF/x64/OVMF_CODE.fd:/usr/share/OVMF/x64/OVMF_VARS.fd"
    "/usr/share/qemu/OVMF_CODE.fd:/usr/share/qemu/OVMF_VARS.fd"
  )
  for pair in "${cands[@]}"; do
    IFS=: read -r c v <<<"$pair"
    if [[ -f "$c" && -f "$v" ]]; then
      OVMF_CODE="$c"
      OVMF_VARS="$v"
      return 0
    fi
  done
  return 1
}

if [[ "$(uname -s)" == "Darwin" ]]; then
  detect_mac || true
else
  detect_linux || true
fi

export OVMF_CODE="${OVMF_CODE:-}"
export OVMF_VARS="${OVMF_VARS:-}"