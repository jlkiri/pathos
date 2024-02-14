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
            "csrs sie, t0",
            // "li t0, 1 << 1",
            // "csrw sstatus, t0",
            in(reg) handler
        )
    }
}

#[naked]
#[no_mangle]
#[repr(align(4))]
fn handle_supervisor_interrupt() {
    unsafe {
        asm!(
            "li t0, 0x10000000",
            "li t1, 0x53",
            "sb t1, (t0)",
            "li t1, 0xa",
            "sb t1, (t0)",
            // "li t0, 1 << 5",
            "csrw sip, zero",
            "csrw scause, zero",
            "csrw stval, zero",
            "li t0, 1 << 8",
            "csrs sstatus, t0",
            "sret",
            options(noreturn)
        )
    }
}

#[no_mangle]
pub extern "C" fn kinit() {
    // uart_print();
    // uart_print_asm(HELLO);

    for char in "kinit\n".chars() {
        unsafe {
            ptr::write_volatile(UART, char as u8);
        }
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

    // unsafe {
    //     asm!(
    //         "li t0, (1 << 5)",
    //         "csrw sie, t0",
    //         "li t0, 1 << 2",
    //         "csrw sstatus, t0"
    //     )
    // }

    unsafe {
        ptr::write_volatile(UART, '0' as u8);
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
