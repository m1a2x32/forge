# Forge OS

**Minimal by default, capable by choice.** Forge is an experimental Rust operating
system. This first milestone boots a small kernel on QEMU's AArch64 `virt` machine
with a Cortex-A53 CPU. It is the starting point for a modular monolithic kernel.

## What runs today

- A real `no_std`, `no_main` AArch64 image, entered at EL1 or EL2 and normalized to EL1.
- A dedicated 64 KiB stack, BSS initialization, and an aligned exception vector table.
- Polled PL011 serial output and fatal exception diagnostics with all 31 registers.
- A portable boot-frame allocator with a platform-selected size, exercised in a
  linker-reserved 64 KiB arena. QEMU currently selects 4 KiB frames.
- Automated QEMU boot, breakpoint, and panic checks, plus host allocator tests.

Execution stops in a wait loop after initialization. There is no scheduler, general
RAM allocator, device-tree parser, MMU setup, unmasked IRQs, userspace or shell yet.
Only the primary CPU runs the kernel. The arena allocator's state is temporary;
its pages remain reserved and no allocated addresses escape the initial check.

## Build and run

On a Debian/Ubuntu development host, install Rust using rustup, a native C linker
for host tests, Python 3, and QEMU's ARM system emulator:

```sh
sudo apt-get update
sudo apt-get install --no-install-recommends build-essential python3 qemu-system-arm
# Install rustup from https://rustup.rs if needed.
rustup show                         # installs the pinned toolchain and target
./scripts/run.sh                    # Ctrl-A X to quit
```

The scripts also work on other Unix hosts with equivalent tools. The pinned Rust
toolchain includes the cross target and LLD linker. There are no crates.io
dependencies, custom Rust targets, nightly features or external objcopy tools.
Set `QEMU_SYSTEM_AARCH64=/path/to/qemu-system-aarch64` for a nonstandard install.

```sh
./scripts/check.sh                  # formatting, unit tests, Clippy and QEMU
./scripts/build.sh                  # build/normal/forge.img and forge.elf
```

## Build configuration

Edit `forge.cfg` in the workspace root:

```ini
boot_frame_size = 4096
```

Use one `name = value` setting per line. Use `#` for comments. This file format
accepts decimal integers. The build rejects missing, duplicate and unknown settings.

The QEMU image accepts a nonzero power of two up to 16384 bytes. This limit leaves
space for the startup allocation check in the fixed 64 KiB boot arena. Other
platforms can use different limits. This setting does not configure the MMU.

To select another file, set `FORGE_CONFIG`:

```sh
FORGE_CONFIG=configs/device.cfg ./scripts/run.sh
```

Create the selected file before you build. Relative paths start at the workspace
root. Absolute paths also work. Cargo reads the file at build time and generates
the Rust constants. Cargo rebuilds when the selected file or `FORGE_CONFIG` changes.
The kernel does not read a configuration file at runtime.

The file currently selects only the frame size. Add settings for optional OS
components when those components are implemented. Test-only Cargo features remain
separate from this device configuration.

Boot output ends with:

```text
Forge OS 0.1.0
memory: boot arena 0x40085000..0x40095000
memory: 16 frames of 4096 bytes, allocation check passed
kernel: initialized
FORGE_BOOT_OK
```

Addresses may move as the image grows. The raw image is about 16 KiB in the initial
release build; stack and boot pages also reserve RAM without occupying image bytes.
Use the normal raw `.img` with QEMU's `-kernel` option. The `.elf` is for symbols
and debugging; passing it directly to `-kernel` selects a different boot protocol.

## Source map

| Location | Responsibility |
| --- | --- |
| `kernel/forge-kernel` | Portable initialization and physical-address allocation; forbids unsafe Rust |
| `arch/forge-arch-aarch64` | CPU entry, EL2 to EL1 transition, register access, exception frames |
| `platforms/forge-platform-qemu-virt` | Memory layout, image entry, PL011, panic and fatal diagnostics |
| `forge.cfg` | Device settings read during the build |
| `scripts` | Cross-build, ELF extraction, emulator launch and automated checks |
| `docs` | Boot contract, implementation lesson and next steps |

UART access stays with the platform until a second platform gives us a concrete
reason for a shared driver crate. OS services will be optional; neither process
isolation nor virtual memory will become a requirement for every future port.

Read [the boot lesson](docs/01-boot.md) for the mechanisms behind this implementation,
[the architecture contract](docs/architecture.md) before changing startup, and
[the next milestone](docs/next.md) for the intended memory-management increment.

For a guided review, read [the commit walkthrough](docs/02-review-walkthrough.md).
[Development conventions](CONTRIBUTING.md) cover commit messages and test files.

This image has no Raspberry Pi firmware integration. QEMU `virt` is a synthetic
board, not an emulation of the Pi Zero 2 W or BCM2710A1 peripherals. Sharing the
Cortex-A53 CPU architecture does not make their board support interchangeable.
