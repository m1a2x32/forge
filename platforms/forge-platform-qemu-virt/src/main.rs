#![no_std]
#![no_main]

mod uart;

use core::{fmt::Write, panic::PanicInfo};
use forge_arch_aarch64 as arch;
use uart::Console;

#[unsafe(no_mangle)]
pub extern "C" fn forge_entry(dtb: usize, entry_el: u64) -> ! {
    let mut console = Console;
    let _ = writeln!(
        console,
        "platform: qemu-virt entry=EL{entry_el} current=EL{}",
        arch::current_el()
    );
    let _ = writeln!(console, "boot: dtb={dtb:#x} (preserved, not parsed yet)");
    let _ = forge_kernel::initialize(&mut console);

    let _ = writeln!(console, "FORGE_BOOT_OK");
    arch::halt()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut console = Console;
    let _ = writeln!(console, "FORGE_PANIC {info}");
    let _ = writeln!(console, "FORGE_PANIC_HALTED");
    arch::halt()
}
