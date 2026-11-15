#![no_std]
#![forbid(unsafe_code)]

pub mod memory;

use core::fmt::{self, Write};

/// Kernel entry after the platform establishes the CPU and console.
pub fn initialize(console: &mut impl Write) -> Result<(), fmt::Error> {
    writeln!(console, "Forge OS 0.1.0")?;
    writeln!(console, "kernel: initialized")
}
