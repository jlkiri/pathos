#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(naked_functions)]

use core::arch::{asm, global_asm};
// use core::f32::consts::PI;
// // use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr;

// use sbi_rt;
// // use core::ptr;

// // global_asm!(include_str!("entry.s"));

mod asm;

const UART: *mut u8 = 0x10000000 as *mut u8;

// #[no_mangle]
#[inline(always)]
fn uart_print_char(char: char) {
    unsafe {
        ptr::write_volatile(UART, char as u8);
    }
}

#[inline(always)]
fn uart_print(s: &str) {
    for char in s.chars() {
        uart_print_char(char);
    }
    uart_print_char('\n');
}

#[inline(always)]
fn setup_interrupt_handlers(dispatcher: fn()) {
    unsafe {
        asm!(
            "csrw stvec, {}",
            "li a0, 1",
            "ecall",
            in(reg) dispatcher
        )
    }
}

enum Cause {
    Interrupt(u8),
    Exception(u8),
}

#[inline]
fn read_scause() -> Cause {
    let scause: i64;
    unsafe {
        asm!(
            "csrr {}, scause",
            out(reg) scause
        )
    }
    uart_print("scause: ");

    let cause = (scause & 0xf) as u8;
    let cause_char = (cause + 48) as u8 as char;
    uart_print_char(cause_char);

    uart_print_char(((scause >> 63 & 0x1) + 48) as u8 as char);

    match scause >> 63 & 0x1 {
        0 => Cause::Exception(cause),
        1 => Cause::Interrupt(cause),
        _ => unreachable!(),
    }
}

#[no_mangle]
#[repr(align(4))]
fn dispatch_smode_interrupt() {
    match read_scause() {
        Cause::Interrupt(5) => {
            uart_print("Software timer interrupt handled.");
            unsafe { asm!("li a0, 2", "ecall", "sret") }
        }
        _ => panic!(),
    }
}

#[no_mangle]
pub extern "C" fn main() {
    uart_print("Entered S-mode handler setup.");

    setup_interrupt_handlers(dispatch_smode_interrupt);

    unsafe { asm!("li t0, 1 << 1", "csrs sstatus, t0", "li a0, 1", "ecall") }

    uart_print("Finished S-mode handler setup.");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart_print("S-mode panic!");
    loop {}
}
