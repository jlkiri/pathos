#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
fn main() {
    // println!("Hello, world!");
}

#[panic_handler]
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
