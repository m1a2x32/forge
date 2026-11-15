#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
./scripts/build.sh
# Ctrl-A X exits QEMU. Pass -S -gdb tcp::1234 to wait for an AArch64-capable GDB.
exec "${QEMU_SYSTEM_AARCH64:-qemu-system-aarch64}" \
  -machine virt,virtualization=on -cpu cortex-a53 -m 128M -smp 1 \
  -display none -serial mon:stdio -nic none -no-reboot \
  -kernel build/normal/forge.img "$@"
