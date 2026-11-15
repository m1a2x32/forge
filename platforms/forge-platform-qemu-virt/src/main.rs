#![no_std]
#![no_main]

mod uart;

use core::{fmt::Write, panic::PanicInfo};
use forge_arch_aarch64 as arch;
use uart::Console;

#[cfg(all(feature = "test-exception", feature = "test-panic"))]
compile_error!("select only one fault-injection feature");

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

    #[cfg(feature = "test-exception")]
    {
        let _ = writeln!(console, "test: injecting BRK");
        arch::breakpoint();
    }
    #[cfg(feature = "test-panic")]
    panic!("intentional panic test");

    #[cfg(not(any(feature = "test-exception", feature = "test-panic")))]
    {
        let _ = writeln!(console, "FORGE_BOOT_OK");
        arch::halt()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn forge_exception(frame: &arch::ExceptionFrame, slot: usize) -> ! {
    // The assembly entry captures a repr(C), 16-byte-aligned stack frame before
    // calling this function. This is fatal-only: no registers are restored yet.
    let mut console = Console;
    let _ = writeln!(
        console,
        "FORGE_EXCEPTION slot={slot} ec={:#x}",
        frame.esr >> 26
    );
    let _ = writeln!(
        console,
        "ELR={:#018x} ESR={:#018x} FAR={:#018x}",
        frame.elr, frame.esr, frame.far
    );
    let _ = writeln!(console, "SP={:#018x} SPSR={:#018x}", frame.sp, frame.spsr);
    for (index, register) in frame.registers.iter().enumerate() {
        let _ = writeln!(console, "x{index:02}={register:#018x}");
    }
    let _ = writeln!(console, "FORGE_EXCEPTION_HALTED");
    arch::halt()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut console = Console;
    let _ = writeln!(console, "FORGE_PANIC {info}");
    let _ = writeln!(console, "FORGE_PANIC_HALTED");
    arch::halt()
}
