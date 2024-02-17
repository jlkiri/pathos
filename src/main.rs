#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(abi_riscv_interrupt)]

use core::arch::asm;
use core::panic::PanicInfo;
use core::{fmt, ptr};

// use interrupts::serial_println;

type HandlerFunc = extern "riscv-interrupt-s" fn();

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

    // serial_println!("scause: {}", cause as u8);

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
            // serial_println!("OK: Software timer interrupt handled.");
            const UART_ADDR: *mut u8 = 0x10000000 as *mut u8;
            unsafe {
                ptr::write_volatile(UART_ADDR as *mut u8, 'b' as u8);
            }
            unsafe { asm!("li x31, 2", "ecall") }
        }
        _ => panic!(),
    }
}

#[no_mangle]
pub extern "C" fn main() {
    const UART_ADDR: *mut u8 = 0x10000000 as *mut u8;
    unsafe {
        ptr::write_volatile(UART_ADDR as *mut u8, 'a' as u8);
    }

    setup_interrupt_handlers(dispatch_smode_interrupt);
    unsafe { asm!("li t0, 1 << 1", "csrs sstatus, t0") }

    // serial_println!("OK: S-mode interrupt handler setup.");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // serial_println!("PANIC: S-mode panic!");
    loop {}
}
