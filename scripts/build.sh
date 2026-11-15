#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mode="${1:-normal}"
features=()
case "$mode" in
  normal) ;;
  exception) features=(--features test-exception) ;;
  panic) features=(--features test-panic) ;;
  *) echo "usage: $0 [normal|exception|panic]" >&2; exit 2 ;;
esac
cargo build --locked --release --target aarch64-unknown-none-softfloat \
  -p forge-platform-qemu-virt "${features[@]}"
mkdir -p "build/$mode"
cp target/aarch64-unknown-none-softfloat/release/forge-qemu-virt "build/$mode/forge.elf"
python3 scripts/elf_to_image.py "build/$mode/forge.elf" "build/$mode/forge.img"
