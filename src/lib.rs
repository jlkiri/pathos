#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(abi_riscv_interrupt)]
#![feature(fn_ptr_trait)]
#![feature(const_mut_refs)]
#![feature(str_from_raw_parts)]
// #![feature(custom_test_frameworks)]

#[cfg(test)]
use core::panic::PanicInfo;
use owo_colors::OwoColorize;

pub mod interrupts;
pub mod serial;

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("PANIC: S-mode panic!");
    loop {}
}

#[no_mangle]
pub fn ok(s: &str) {
    let pathos_prelude = "PathOS".blue();
    let ok = "OK".green();
    serial_print!("{} ", pathos_prelude);
    serial_print!("[{}] ", ok);
    serial_println!("{}", s);
}

#[no_mangle]
pub fn ok_raw(ptr: *const u8, len: usize) {
    ok(unsafe { core::str::from_raw_parts(ptr, len) })
}
