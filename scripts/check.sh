#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --all -- --check
cargo test --locked -p forge-kernel
mkdir -p build
rustc --edition=2024 --test platforms/forge-platform-qemu-virt/build_config.rs \
  -o build/build-config-tests
./build/build-config-tests
cargo clippy --locked -p forge-kernel --all-targets -- -D warnings
cargo clippy --locked --target aarch64-unknown-none-softfloat \
  -p forge-platform-qemu-virt -- -D warnings
python3 scripts/config_test.py
python3 scripts/smoke.py
