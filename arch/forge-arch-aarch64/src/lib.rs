#![no_std]

#[cfg(not(target_arch = "aarch64"))]
compile_error!("forge-arch-aarch64 must be cross-compiled for AArch64");

use core::arch::{asm, global_asm};

// Retained by the platform linker script. This crate owns CPU startup;
// the platform supplies memory layout symbols and the Rust entry function.
global_asm!(include_str!("boot.S"));

pub fn current_el() -> u64 {
    let value: u64;
    // SAFETY: read-only CPU register available at our EL1 execution level.
    unsafe { asm!("mrs {}, CurrentEL", out(reg) value, options(nomem, nostack, preserves_flags)) };
    value >> 2
}

/// Fatal/idle stop for this milestone. All interrupts remain masked, and there is
/// no scheduler or wakeup protocol yet. WFE can wake spuriously, so always loop.
pub fn halt() -> ! {
    loop {
        // SAFETY: waiting changes no Rust memory and cannot expose an interrupt
        // handler: startup masks DAIF and this milestone never unmasks it.
        unsafe { asm!("wfe", options(nomem, nostack)) };
    }
}
