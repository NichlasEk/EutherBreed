#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."

if [ "${1:-}" = "--gl" ]; then
  shift
  export WGPU_BACKEND=gl
  echo "Using WGPU_BACKEND=gl." >&2
fi

if ! command -v nvidia-smi >/dev/null 2>&1; then
  echo "nvidia-smi not found; trying GUI run anyway." >&2
elif ! nvidia-smi >/dev/null 2>&1; then
  echo "NVIDIA driver check failed. GUI may not start until the driver stack is fixed." >&2
  echo "Use scripts/headless-smoke.sh for non-GPU verification." >&2
fi

cargo run -p euther_game -- "$@"
