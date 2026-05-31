#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."

cargo fmt --check
cargo check
cargo test -p game_core
cargo run -p abta_tools -- --help
cargo run -p euther_game -- --validate-content
cargo run -p euther_game -- --headless-smoke
