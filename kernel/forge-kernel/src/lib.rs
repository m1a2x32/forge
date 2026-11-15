#![no_std]
#![forbid(unsafe_code)]

pub mod memory;

use core::fmt::{self, Write};
use memory::BootFrameAllocator;

/// Kernel entry after the platform establishes the CPU, console and boot arena.
/// The arena must be reserved from all other allocators. No memory is dereferenced
/// here: this first allocator only hands out physical page addresses.
pub fn initialize<const FRAME_SIZE: usize>(
    console: &mut impl Write,
    arena_start: usize,
    arena_end: usize,
) -> Result<(), fmt::Error> {
    writeln!(console, "Forge OS 0.1.0")?;
    let mut frames = BootFrameAllocator::<FRAME_SIZE>::new(arena_start, arena_end)
        .expect("platform supplied an invalid boot arena");
    let initial_pages = frames.remaining();
    let first = frames.allocate().expect("boot arena is empty");
    let second = frames.allocate().expect("boot arena needs two pages");
    assert_eq!(second.address(), first.address() + first.size());
    assert_eq!(frames.remaining(), initial_pages - 2);
    writeln!(
        console,
        "memory: boot arena {arena_start:#x}..{arena_end:#x}"
    )?;
    writeln!(
        console,
        "memory: {initial_pages} frames of {FRAME_SIZE} bytes, allocation check passed"
    )?;
    writeln!(console, "kernel: initialized")
    // The temporary allocator is intentionally discarded. No addresses escape;
    // the entire arena stays reserved. General RAM management is a later step.
}
