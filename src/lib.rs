#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(abi_riscv_interrupt)]
// #![feature(custom_test_frameworks)]

pub mod serial;

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("PANIC: S-mode panic!");
    loop {}
}
