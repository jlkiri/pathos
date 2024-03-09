#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
fn main() -> u32 {
    1 // Must return a value
}

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
