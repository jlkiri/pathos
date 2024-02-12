#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::f32::consts::PI;
// use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr;
use sbi_rt;
// use core::ptr;

// global_asm!(include_str!("entry.s"));

mod asm;

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

fn setup_interrupt_table(handler: fn()) {
    unsafe {
        asm!(
            "csrw stvec, {}",
            "li       t0, (1 << 2) | (1 << 8)",
            "csrw sstatus, t0",
            in(reg) handler
        )
    }
}

fn handle_supervisor_interrupt() {
    // unsafe {
    //     ptr::write(UART, 'X' as u8);
    // }
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    // uart_print();
    // uart_print_asm(HELLO);
    const UART: *mut u8 = 0x10000000 as *mut u8;

    for c in "hello".chars() {
        unsafe {
            ptr::write_volatile(UART, c as u8);
        }
    }

    // setup_interrupt_table(handle_supervisor_interrupt);
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
