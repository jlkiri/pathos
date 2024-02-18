#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(abi_riscv_interrupt)]
#![feature(fn_ptr_trait)]
#![feature(const_mut_refs)]
#![feature(str_from_raw_parts)]
#![feature(asm_const)]
// #![feature(custom_test_frameworks)]

#[cfg(test)]
use core::panic::PanicInfo;

pub mod interrupts;
pub mod serial;

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    error!("S-mode kernel panic!");
    loop {}
}

#[no_mangle]
pub fn mprint(ptr: *const u8, len: usize) {
    let s = unsafe { core::str::from_raw_parts(ptr, len) };
    // serial_info(s)
}
