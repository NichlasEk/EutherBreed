#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."

cargo fmt --check
cargo check
cargo test -p game_core
cargo run -p abta_tools -- --help
cargo run -p euther_game -- --validate-content
cargo run -p euther_game -- --save-smoke
cargo run -p euther_game -- --save-file-smoke /tmp/euther_save_smoke.ron
cargo run -p euther_game -- --load-file-smoke /tmp/euther_save_smoke.ron
cargo run -p euther_game -- --runtime-save-smoke /tmp/euther_runtime_save_smoke.ron
rm -f /tmp/euther_save_smoke.ron
rm -f /tmp/euther_runtime_save_smoke.ron
cargo run -p euther_game -- --headless-smoke
