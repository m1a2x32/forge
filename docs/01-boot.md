# Lesson 1: giving Rust a machine it can run on

## The first kernel responsibility

A normal Rust program inherits an initialized process from its OS. A kernel has
to establish its own execution environment. Before calling even a small Rust
function, Forge needs a valid stack, known privilege state, correctly initialized
static storage and a way to explain failures.

Start reading at `_start` in `arch/forge-arch-aarch64/src/boot.S`. Its first 64 bytes
are an ARM64 image header, beginning with a branch to the real startup code. QEMU
uses the header to load the raw image and supply a device-tree pointer in x0. We
save that pointer before reusing registers for CPU setup.

## Why EL1?

AArch64 uses exception levels. EL0 is typically application execution; EL1 is the
kernel; EL2 supports hypervisors. QEMU can start us at EL2 when virtualization is
enabled. Rust itself does not normalize those levels for us.

The EL2 branch configures EL1 for AArch64 execution, assigns its stack, sets the
return address and SPSR, then executes `eret`. This instruction restores the
selected state and enters EL1h. When QEMU already starts at EL1, we use the common
EL1 path directly. No hypervisor remains running behind the kernel.

`DAIF` masks debug, asynchronous abort, IRQ and FIQ exceptions during startup.
Synchronous faults such as invalid instructions are still possible. Masking IRQs
does not replace having exception vectors.

## What the linker does

`platforms/forge-platform-qemu-virt/linker.ld` places code at the address QEMU uses.
It also names the BSS interval, stack and boot-page arena. Those linker symbols
are addresses, not Rust variables holding addresses: `addr_of!` obtains the symbol
address without reading an imaginary value from it.

Loaded `.data` bytes come from the image. Zero-initialized `.bss` consumes memory
without storing all those zeros in the image file. Our assembly loop explicitly
clears BSS. It must not clear the live stack. The boot stack is separately reserved
and aligned to the AArch64 calling convention's 16-byte boundary.

The raw file is small partly because the 64 KiB stack and 64 KiB page arena are
reservations rather than file contents. A 16 KiB image therefore does not mean the
whole running system occupies only 16 KiB of RAM.

## Talking to the machine

QEMU `virt` puts a PL011 UART at physical address `0x09000000`. With translation
disabled we access that address directly. Volatile reads check whether its TX
FIFO is full; volatile writes enqueue bytes. Ordinary memory accesses could be
optimized or combined in ways unsuitable for hardware registers.

The driver implements `core::fmt::Write`, letting kernel code print without a
heap or libc. We rely on QEMU's UART model accepting this early-console use; a
real board will need clock and UART initialization in its own platform layer.

## Making failures observable

VBAR_EL1 points to a vector table with 16 possible entry slots. The processor
selects a slot from exception type and source execution state. For our injected
BRK at EL1h it chooses slot 4, the synchronous exception handler for the current
EL using SP_EL1.

Assembly saves registers before calling Rust. This matters because a normal
function call can overwrite registers containing the evidence of what failed.
The handler reports the exception syndrome in ESR_EL1 and faulting instruction
address in ELR_EL1. BRK reports exception class 0x3c. These fatal handlers stop;
recoverable exceptions and IRQs will require a register-restore and return path.

## The first memory mechanism

`BootFrameAllocator<FRAME_SIZE>` owns a cursor and an exclusive end address. Each
allocation advances the cursor by the platform's configured size (4096 bytes on
the current QEMU image). A returned `PhysicalFrame<FRAME_SIZE>` reports both its
address and its size. It is useful for structures we will keep throughout
boot, such as page tables. A free list or bitmap allocator will be needed for
reusable frames later. The current allocator only describes addresses and does
not initialize, map or access the pages themselves.

The tradeoff is deliberate: a bump allocator is easy to audit and needs no dynamic
metadata, but cannot free memory or skip reserved holes. We give it a tiny known
arena instead of guessing which RAM is free. Discovering and reserving real RAM
is the next milestone.

## Try it

Run `./scripts/run.sh`, observe `FORGE_BOOT_OK`, and exit with Ctrl-A X. Then run
`./scripts/check.sh` to exercise both boot levels and fault paths automatically.
UART transcripts are saved under `build/logs`.

To debug with an AArch64-capable GDB, start `./scripts/run.sh -S -gdb tcp::1234`,
open `build/normal/forge.elf` in GDB, connect with `target remote :1234`, set
`break forge_entry`, and continue. The raw image runs while the ELF supplies symbols.
