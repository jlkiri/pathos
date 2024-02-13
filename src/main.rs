#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(naked_functions)]

use core::arch::{asm, global_asm};
use core::f32::consts::PI;
// use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr;
use sbi_rt;
// use core::ptr;

// global_asm!(include_str!("entry.s"));

mod asm;

const UART: *mut u8 = 0x10000000 as *mut u8;

// const HELLO: &str = "HELLO";

// // #[no_mangle]
// #[inline(always)]
// fn uart_print_asm(s: &str) {
//     unsafe {
//         asm!(
//             "li a7, 0x4442434E",
//             "li a6, 0x00",
//             "li a0, 11",
//             "li a2, 0",
//             "ecall",
//             in("a1") s.as_ptr(),
//         );
//     }
// }

#[inline]
fn setup_interrupt_table(handler: fn()) {
    unsafe {
        asm!(
            "csrw stvec, {}",
            "li t0, (1 << 5)",
            "csrw sie, t0",
            in(reg) handler
        )
    }
}

#[naked]
#[repr(align(4))]
fn handle_supervisor_interrupt() {
    unsafe {
        // asm!(".align 4");
        // ptr::write_volatile(UART, 'X' as u8);
        asm!("sret", options(noreturn))
    }
}

#[no_mangle]
pub extern "C" fn kinit() {
    // uart_print();
    // uart_print_asm(HELLO);

    unsafe {
        ptr::write_volatile(UART, 'B' as u8);
    }

    // loop {}
}

#[no_mangle]
pub extern "C" fn main() {
    // uart_print();
    // uart_print_asm(HELLO);
    const UART: *mut u8 = 0x10000000 as *mut u8;

    // unsafe {
    //     ptr::write_volatile(UART, 'B' as u8);
    // }

    // setup_interrupt_table(handle_supervisor_interrupt);

    setup_interrupt_table(handle_supervisor_interrupt);

    unsafe {
        ptr::write_volatile(UART, 'M' as u8);
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        ptr::write_volatile(UART, 'P' as u8);
    }

    loop {}
}
