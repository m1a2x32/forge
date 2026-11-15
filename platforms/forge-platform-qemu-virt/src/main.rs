#![no_std]
#![no_main]

mod config;
mod uart;

use core::{fmt::Write, panic::PanicInfo, ptr};
use forge_arch_aarch64 as arch;
use uart::Console;

#[cfg(all(feature = "test-exception", feature = "test-panic"))]
compile_error!("select only one fault-injection feature");

unsafe extern "C" {
    static __boot_pages_start: u8;
    static __boot_pages_end: u8;
    static __stack_bottom: u8;
    static __stack_top: u8;
}

// Force real memory reads to check both zero-initialized and loaded data in QEMU.
#[used]
#[unsafe(link_section = ".bss.boot_probe")]
static ZERO_PROBE: [u64; 8] = [0; 8];
#[used]
#[unsafe(link_section = ".data.boot_probe")]
static DATA_PROBE: u64 = 0x464f_5247_454f_5321;

#[unsafe(no_mangle)]
pub extern "C" fn forge_entry(dtb: usize, entry_el: u64) -> ! {
    let mut console = Console;
    let _ = writeln!(
        console,
        "platform: qemu-virt entry=EL{entry_el} current=EL{}",
        arch::current_el()
    );
    assert_eq!(arch::current_el(), 1);
    assert_eq!(arch::system_control() & ((1 << 12) | (1 << 2) | 1), 0);
    let sp = arch::stack_pointer();
    assert_eq!(sp % 16, 0);
    assert!(sp > ptr::addr_of!(__stack_bottom) as usize);
    assert!(sp <= ptr::addr_of!(__stack_top) as usize);
    // SAFETY: these are valid static objects initialized by the loader/BSS loop.
    // Volatile reads ensure checks inspect actual target RAM, not folded constants.
    unsafe {
        for index in 0..8 {
            assert_eq!(
                ptr::read_volatile(ptr::addr_of!(ZERO_PROBE).cast::<u64>().add(index)),
                0
            );
        }
        assert_eq!(
            ptr::read_volatile(ptr::addr_of!(DATA_PROBE)),
            0x464f_5247_454f_5321
        );
    }
    assert_ne!(dtb, 0, "direct-kernel boot did not supply a device tree");
    let _ = writeln!(
        console,
        "boot: BSS/data/stack checks passed; MMU/caches off"
    );
    let _ = writeln!(console, "boot: dtb={dtb:#x} (preserved, not parsed yet)");
    let arena_start = ptr::addr_of!(__boot_pages_start) as usize;
    let arena_end = ptr::addr_of!(__boot_pages_end) as usize;
    let _ = forge_kernel::initialize::<{ config::BOOT_FRAME_SIZE }>(
        &mut console,
        arena_start,
        arena_end,
    );

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
