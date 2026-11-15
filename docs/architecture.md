# Initial execution contract

## Scope

Forge starts with one portable kernel, architecture-specific CPU mechanisms and
platform-specific wiring. All code currently runs privileged in a single address
space, which is a useful small monolithic starting point. The absence of task
isolation here does not decide the eventual application model on other CPUs.

No optional services run in the background. Startup requires no heap. Allocator
metadata is just two machine-sized addresses on the stack.

## QEMU boot contract

Run `qemu-system-aarch64 -machine virt,virtualization=on -cpu cortex-a53 -m 128M`
with `-kernel build/normal/forge.img`. The nonvirtualized machine is also tested.
This is actual AArch64 target code running in QEMU's CPU and peripheral models,
not the portable kernel recompiled as a Linux executable.

The image uses the documented ARM64 Image header format and QEMU's direct-kernel
boot convention: entry at `0x40080000`, DTB address in x0, MMU and caches off, EL1
or EL2, and primary CPU affinity zero. The header describes an executable image;
it does not introduce Linux binary or syscall compatibility.

Startup masks DAIF, parks directly entered secondary affinities, preserves x0,
and drops EL2 to EL1 when required. EL2 enables EL1 access to physical counters
but no timer is programmed yet. EL1 uses SP_EL1 and a 16-byte-aligned 64 KiB stack.
It sets SCTLR_EL1 for the ARMv8-A baseline with M/C/I disabled, zeros BSS, installs
VBAR_EL1, and calls Rust. Arbitrary firmware with an active MMU/cache, EL3 entry,
UEFI, and physical-board startup are outside this contract.

The kernel is compiled for `aarch64-unknown-none-softfloat`: generated Rust does
not use SIMD/floating-point registers. No FP context is enabled or saved. Enabling
FP later requires explicit exception and task-context changes.

## Memory ownership

The linker controls text, read-only data, loaded data, BSS, a 64 KiB boot-page
arena and a separate 64 KiB stack. Text, rodata and data have distinct ELF segment
flags for inspection; with the MMU off these flags do **not** enforce protection.
The first 512 KiB of QEMU RAM below the image is left alone. An assertion keeps
the whole initial reservation below `0x40200000` (under 2 MiB above RAM base).

The BSS loop clears only BSS, not the live stack or page arena. The allocator
never touches page contents. Its caller supplies an exclusive, already reserved
half-open interval. Aligning inward prevents handing out partial pages; checked
arithmetic rejects a start address whose alignment would wrap.

`BootFrameAllocator<FRAME_SIZE>` takes its nonzero power-of-two allocation size
from platform build configuration. `forge.cfg` selects 4096 for the QEMU image.
`FORGE_CONFIG` can select another file. The platform build script validates the
file and generates the constants included by `src/config.rs`. Cargo tracks changes
to the file and its selection. No parser is included in the kernel image.

The QEMU build accepts powers of two up to 16384 bytes. This limit guarantees at
least two whole frames in its 64 KiB arena after alignment. Larger frames require
a change to the platform reservation and its validation limit. The portable
allocator supports other sizes, including smaller MCU allocation units.

`PhysicalFrame<FRAME_SIZE>` exposes `address()` and `size()`. Its size is part of
the Rust type, so it does not take up a second machine word in every descriptor.
Allocator fields `next_frame_address` and `region_end_address` are physical byte
addresses. The latter is an exclusive bound, and the former can reach that bound
at exhaustion; neither is itself an allocated frame.

This configuration does not set a hardware translation granule. When MMU support
arrives it must validate the CPU's supported sizes, page-table format, mapping
alignment and attributes. Linker section alignment is also a separate constraint;
changing the allocator setting does not change the linker script.

`PhysicalFrame<FRAME_SIZE>` is an address descriptor, not a safe pointer or a mapping. The
allocator does not enforce system-wide exclusivity: two independently constructed
allocators could describe overlapping regions. Any future code dereferencing
frames must prove ownership, reachability and mapping separately. Boot allocator
state is not shared between CPUs or interrupt handlers; it has no locking.

Today only the linker arena is passed to it, and no allocations escape the
initialization check. No claim is made that the remaining QEMU RAM is allocatable.
The DTB pointer is preserved and logged but never dereferenced or parsed yet.

## Exceptions and diagnostics

All 16 architectural vector slots exist at 128-byte spacing inside a 2 KiB-aligned
table. Each saves original x0/x1, attaches its slot index, and enters common code.
That code captures all x0–x30, original handler-stack SP, ELR, ESR, FAR and SPSR in
a 288-byte `repr(C)` frame. Rust emits the frame through PL011 and halts.

These are fatal-only handlers, not context switches. There is no `eret` return,
interrupt-controller initialization, nested-fault recovery or dedicated exception
stack. A damaged/overflowed active stack can still prevent diagnosis. For future
exceptions from EL0, the captured SP is the handler's original SP_EL1, not SP_EL0.
FAR is printed for inspection but is meaningful only for applicable exception types.

Only the primary core writes the console, all interrupts remain masked, and MMIO
uses aligned volatile raw-pointer accesses. Panic and fatal handlers construct a
fresh console with no lock, so they cannot deadlock on an interrupted print lock.
This single-core design must be revisited before concurrent logging is enabled.

## Verification boundary

`cargo test -p forge-kernel` tests portable allocator arithmetic on the host.
It is not hardware SIL. `scripts/smoke.py` separately executes the actual ARM64
image, checks initial and current EL, BSS/data/stack checks, allocation diagnostics,
and boot completion. It tests EL1 and EL2 with 1 CPU/128 MiB and 4 CPUs/256 MiB.
The four-CPU cases check for one primary initialization; QEMU's PSCI holds the
other CPUs off, so they do not validate the defensive secondary parking branch.

Fault images inject BRK and panic at both entry levels. BRK must select EL1h's
synchronous slot 4, report EC 0x3c and immediate 0x46, and preserve seeded register
values. Both fatal paths must reach their final diagnostics without boot success.
Timeouts fail; the harness then terminates QEMU after checking the expected output.
No semihosting exit or debug device is required in the production image.

This does not validate real hardware, interrupt delivery, allocation of general
RAM, memory protection, peripheral timing, SMP execution or recovery from faults.
