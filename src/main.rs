#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(abi_riscv_interrupt)]

use core::arch::asm;
use core::panic::PanicInfo;
use core::{fmt, ptr};

mod asm;

type HandlerFunc = extern "riscv-interrupt-s" fn();

const UART_ADDR: usize = 0x10000000;
static mut SERIAL: Serial = Serial(UART_ADDR as *mut u8);

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        SERIAL.write_fmt(args).expect("Printing to serial failed");
    }
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

struct Serial(*mut u8);

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            unsafe { ptr::write_volatile(self.0, byte) }
        }
        Ok(())
    }
}

#[inline(always)]
fn uart_print_char(char: char) {
    unsafe {
        ptr::write_volatile(UART_ADDR as *mut u8, char as u8);
    }
}

#[inline(always)]
fn uart_println(s: &str) {
    for char in s.chars() {
        uart_print_char(char);
    }
    uart_print_char('\n');
}

#[inline(always)]
fn setup_interrupt_handlers(dispatcher: HandlerFunc) {
    unsafe {
        asm!(
            "csrw stvec, {}",
            "li x31, 1",
            "ecall",
            in(reg) dispatcher
        )
    }
}

enum Cause {
    Interrupt(u8),
    Exception(u8),
}

#[inline(always)]
fn read_scause() -> Cause {
    let scause: u64;
    unsafe {
        asm!(
            "csrr {}, scause",
            out(reg) scause
        )
    }

    let cause = scause as i64;

    serial_println!("scause: {}", cause as u8);

    match cause.signum() {
        1 => Cause::Exception(cause as u8),
        -1 => Cause::Interrupt(cause as u8),
        _ => unreachable!(),
    }
}

// #[no_mangle]
#[repr(align(4))]
extern "riscv-interrupt-s" fn dispatch_smode_interrupt() {
    match read_scause() {
        Cause::Interrupt(5) => {
            uart_println("OK: Software timer interrupt handled.");
            unsafe { asm!("li x31, 2", "ecall") }
        }
        _ => panic!(),
    }
}

#[no_mangle]
pub extern "C" fn main() {
    setup_interrupt_handlers(dispatch_smode_interrupt);
    unsafe { asm!("li t0, 1 << 1", "csrs sstatus, t0") }

    uart_println("OK: S-mode interrupt handler setup.");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart_println("PANIC: S-mode panic!");
    loop {}
}
