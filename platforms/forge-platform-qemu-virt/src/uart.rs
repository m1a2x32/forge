//! Polling PL011 on QEMU virt. QEMU supplies a configured UART on direct boot.
//! No baud/clock setup is claimed for physical boards. TX only, no IRQs or heap.

use core::{fmt, ptr};

const UART_BASE: usize = 0x0900_0000;
const UART_DR: *mut u32 = UART_BASE as *mut u32;
const UART_FR: *const u32 = (UART_BASE + 0x18) as *const u32;
const TX_FIFO_FULL: u32 = 1 << 5;

pub struct Console;

impl Console {
    fn put_byte(&mut self, byte: u8) {
        // SAFETY: the QEMU virt memory map defines aligned 32-bit PL011 registers
        // here. MMU is off. Only the primary core executes, interrupts are masked,
        // and raw volatile accesses create no Rust references to device memory.
        unsafe {
            while ptr::read_volatile(UART_FR) & TX_FIFO_FULL != 0 {
                core::hint::spin_loop();
            }
            ptr::write_volatile(UART_DR, u32::from(byte));
        }
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            if byte == b'\n' {
                self.put_byte(b'\r');
            }
            self.put_byte(byte);
        }
        Ok(())
    }
}
