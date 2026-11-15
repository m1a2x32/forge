#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mode=normal
cargo build --locked --release --target aarch64-unknown-none-softfloat \
  -p forge-platform-qemu-virt
mkdir -p "build/$mode"
cp target/aarch64-unknown-none-softfloat/release/forge-qemu-virt "build/$mode/forge.elf"
python3 scripts/elf_to_image.py "build/$mode/forge.elf" "build/$mode/forge.img"
