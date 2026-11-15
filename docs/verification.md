# Verification of the first milestone

Verified on 2026-11-15 using Rust 1.90.0 (1159e78c4) and QEMU 10.0.13 on an x86-64
Debian 13 host. Target: `aarch64-unknown-none-softfloat`, release profile, emulated
Cortex-A53 on `virt`. No physical board was used.

`./scripts/check.sh` completed successfully:

| Check | Result |
| --- | --- |
| Rust formatting | Passed |
| Portable allocator tests | 7 passed; includes 256-byte, 4 KiB, 16 KiB and 64 KiB allocation units |
| Configuration parser tests | 6 passed; covers syntax, required settings and platform limits |
| Configuration build checks | File edits produce 4 KiB and 16 KiB images; invalid and missing files stop the build |
| Host Clippy, including tests, warnings denied | Passed |
| AArch64 Clippy, warnings denied | Passed |
| Cross-build normal, exception and panic images | Passed |
| EL1 entry, 1 CPU, 128 MiB | Boot passed |
| EL1 entry, 4 CPUs, 256 MiB | One primary boot passed |
| EL2 entry, 1 CPU, 128 MiB | Transition to EL1 and boot passed |
| EL2 entry, 4 CPUs, 256 MiB | Transition to EL1 and one primary boot passed |
| BRK from EL1 and EL2 boot configurations | 2 passed; slot, syndrome and register capture checked |
| Panic from EL1 and EL2 boot configurations | 2 passed; complete fatal output checked |

The configuration build test selects a temporary `.cfg` file through `FORGE_CONFIG`.
It first sets 4096 bytes per frame, then changes the same file to 16384 bytes.
Each build boots through EL1 and EL2 and reports the expected frame size. Thus,
the test also checks that Cargo rebuilds after a configuration file changes.
Invalid values and missing files stop the build. The test then restores the
caller's configuration. These checks validate allocation settings; the MMU is off.

The normal raw image is 16,392 bytes. The Image header reserves 151,552 bytes
(148 KiB) through the end of the stack, including the 64 KiB boot arena and 64 KiB
stack. These are initial milestone measurements, not long-term budgets or a count
of all emulated RAM. No heap or background services are present.

Boot logs end in `FORGE_BOOT_OK`. BRK logs end in `FORGE_EXCEPTION_HALTED` after
reporting `ESR=0x00000000f2000046`, EC 0x3c, vector slot 4 and all x0–x30. Seeded
x0, x1, x2, x16 and x30 values are asserted by the harness. Panic logs contain the
intentional panic message and end in `FORGE_PANIC_HALTED`. Fault images must not
report successful boot.

Fresh checks produce UART logs under `build/logs/` and images with their debug-symbol
ELFs under `build/`. Generated binaries and logs are excluded from Git. GitHub
Actions runs the same checks and uploads UART logs; its status is reported separately
on the pull request. The results above describe the local verification run.

RAM discovery, page mapping, IRQ/timer delivery, task switching, concurrency and
physical hardware remain unimplemented and unverified. See `architecture.md` for
the complete initial execution contract and `next.md` for the next increment.
