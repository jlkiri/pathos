#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr;

global_asm!(include_str!("entry.s"));

const HELLO: &str = "HELLO";

fn uart_print() {
    let bytes = sbi_rt::Physical::new(5, HELLO.as_ptr() as usize, 0);
    sbi_rt::console_write(bytes);
}

fn uart_print_asm(s: &str) {
    unsafe {
        asm!(
            "li a7, 0x4442434E",
            "li a6, 0x00",
            "li a0, 11",
            "li a2, 0",
            "ecall",
            in("a1") s.as_ptr(),
        );
    }
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    // uart_print();
    uart_print_asm(HELLO);
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
