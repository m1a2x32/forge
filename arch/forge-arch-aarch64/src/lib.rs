#![no_std]

#[cfg(not(target_arch = "aarch64"))]
compile_error!("forge-arch-aarch64 must be cross-compiled for AArch64");

use core::arch::{asm, global_asm};

// Retained by the platform linker script. This crate owns CPU startup and vectors;
// the platform supplies memory layout symbols and the two Rust entry functions.
global_asm!(include_str!("boot.S"));
global_asm!(include_str!("exceptions.S"));

#[repr(C)]
pub struct ExceptionFrame {
    pub registers: [u64; 31],
    pub sp: u64,
    pub elr: u64,
    pub esr: u64,
    pub far: u64,
    pub spsr: u64,
}

const _: () = assert!(core::mem::size_of::<ExceptionFrame>() == 288);

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

/// Deliberately enter the synchronous exception vector to test diagnostics.
pub fn breakpoint() -> ! {
    // SAFETY: the platform has installed fatal exception vectors on a valid stack.
    // BRK enters the handler, which never returns.
    unsafe {
        asm!(
            "mov x0, #0x100",
            "mov x1, #0x101",
            "mov x2, #0x102",
            "mov x16, #0x110",
            "mov x30, #0x11e",
            "brk #0x46",
            options(noreturn)
        )
    };
}
