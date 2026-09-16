#!/bin/sh
set -eu
project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_root"
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
python3 -m unittest discover -s tests -p 'test_*.py'
cargo audit --deny warnings --json > target/security-audit.json
cargo build --release --locked
python3 scripts/generate-docs.py --check
printf 'Source release checks passed. Public binary distribution also requires Developer ID signing and notarization.\n'
