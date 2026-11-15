# Reading the commits after the allocator

The allocator commit (**Add boot page allocation**) adds portable
bookkeeping. Given a reserved address interval, it hands out successive frame
addresses. It does not start a CPU, touch the returned memory, or enable an MMU.
The following commits connect that mechanism to a running ARM64 image.

The table describes the seven logical commits. Each commit includes its review
corrections. Use the commit titles to find them after a history change.

| Commit | What changes | What that lets us observe |
| --- | --- | --- |
| Add AArch64 kernel boot | CPU entry, stack/BSS setup, linker layout, UART and build/run scripts | QEMU executes Rust and prints `FORGE_BOOT_OK` |
| Add fatal exception diagnostics | Vector table, saved CPU state, fatal handler and fault-injection image variants | A deliberate fault reports where and why it happened |
| Add boot initialization checks | Assertions about CPU/memory state; a reserved arena exercises the allocator | Boot success also verifies those startup assumptions |
| Add automated kernel checks | Host tests, cross-builds, QEMU output checks and CI | A repeatable command catches regressions |
| Document the boot milestone | Boot lesson, architecture contract and verification record | The code's assumptions and limits are available for review |

## 1. Getting from reset into Rust

Read [boot.S](../arch/forge-arch-aarch64/src/boot.S) alongside the platform's
[linker script](../platforms/forge-platform-qemu-virt/linker.ld). QEMU loads the
image at the address the linker chose and begins at `_start`. Startup assembly
provides the environment an ordinary executable would inherit from its OS:

1. Mask asynchronous exceptions and keep execution on the primary CPU.
2. Preserve QEMU's device-tree pointer for later hardware discovery.
3. Enter EL1, the ARM kernel privilege level. If QEMU starts at EL2, configure
   EL1 and use `eret` to enter it. EL2 is a higher privilege level, not a second CPU.
4. Set the stack pointer to reserved RAM, and clear BSS (zero-initialized statics).
5. In the final implementation, install exception vectors before entering Rust.
6. Call the platform's Rust function `forge_entry`.

The linker is a build-time tool that assigns addresses to code, data and reserved
memory. It does not execute at boot. Similarly, `build.rs` passes the linker script
to the build, and `elf_to_image.py` produces the raw image QEMU loads.

In [main.rs](../platforms/forge-platform-qemu-virt/src/main.rs), `forge_entry`
uses [uart.rs](../platforms/forge-platform-qemu-virt/src/uart.rs) to print. PL011 is
QEMU's serial device. With the MMU off, its physical register addresses are used
directly. The `core::fmt::Write` interface lets portable kernel code print through
that console without knowing PL011's addresses.

## 2. Capturing failures before the evidence is lost

ARM sends CPU exceptions to specific slots in the table in
[exceptions.S](../arch/forge-arch-aarch64/src/exceptions.S). The handler must save
registers in assembly before calling Rust, because normal function calls can
overwrite them. The captured values are passed to `forge_exception`, which prints
them and waits forever.

Two different meanings of "frame" appear here:

- `PhysicalFrame` describes a fixed-size block of physical memory.
- `ExceptionFrame` contains a snapshot of CPU registers on the existing stack.

The exception code does not ask the physical allocator for memory. Its stack space
was reserved by the linker. This milestone diagnoses fatal exceptions; returning
from an interrupt will require restoring CPU state and executing `eret`.

The `test-exception` build deliberately executes `BRK`, a breakpoint instruction.
The `test-panic` build deliberately triggers Rust's panic handler. These build
variants allow automated checks of both diagnostic paths. Normal boot selects
neither variant.

## 3. Checking the foundation before using it

Startup assertions confirm that execution reached EL1, the stack is within its
reserved range and aligned, BSS contains zeros, loaded data contains the expected
value, and the MMU/caches are still disabled. These are boot-time checks, not
continuously running services.

The linker also reserves a separate 64 KiB arena. `forge_kernel::initialize`
constructs an allocator over it, takes two frames and checks their spacing and
the remaining count. This exercises the allocator on the target CPU. The
allocator is then discarded; those addresses do not escape and the arena stays
reserved. No other kernel subsystem consumes those allocations yet.

The frame size comes from `forge.cfg`. At build time, the platform's `build.rs`
reads that file and generates a Rust constant. `src/config.rs` includes the
generated file. The image therefore contains the selected value and no parser.

On success the console prints `FORGE_BOOT_OK` and execution enters a `WFE` loop.
There are no tasks, scheduling or timer interrupts yet. The next milestone is
discovering usable RAM and reservations from the device tree, so allocation can
extend beyond the small known arena.

## 4. Repeating those checks

`scripts/check.sh` runs portable allocator tests on the development host and then
`scripts/smoke.py` runs actual ARM64 images in QEMU. The latter checks UART output
for normal boot, breakpoint and panic behavior through EL1 and EL2 entry paths.
The four-CPU configurations still run only the primary kernel CPU. They do not
demonstrate multicore scheduling.

Read in this order: `boot.S` and `linker.ld`, then `forge_entry` and `uart.rs`, then
`forge_kernel::initialize`, then `exceptions.S` and `forge_exception`. Review the
automation after the behavior it observes is clear.
