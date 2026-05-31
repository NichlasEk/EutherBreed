#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."

save_path="${1:-/tmp/euther_save_smoke.ron}"
cargo run -p euther_game -- --save-file-smoke "$save_path"
