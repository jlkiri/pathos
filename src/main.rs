#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(abi_riscv_interrupt)]
#![feature(fn_ptr_trait)]
#![feature(const_mut_refs)]

mod asm;

use core::arch::asm;
use core::panic::PanicInfo;
use lazy_static::lazy_static;

use pathos::interrupts::{self, InterruptIndex, InterruptVectorTable};
use pathos::serial_info;

lazy_static! {
    static ref IVT: InterruptVectorTable = {
        let mut ivt = InterruptVectorTable {};
        ivt.register_handler(
            InterruptIndex::SupervisorTimer,
            interrupts::dispatch_smode_interrupt,
        );
        ivt
    };
}

#[no_mangle]
pub fn main() {
    serial_info!("Enter supervisor mode boot setup");

    IVT.init();

    unsafe { asm!("li t0, 1 << 1", "csrs sstatus, t0") }

    serial_info!("Setup interrupt vector table");

    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("PANIC: S-mode panic!");
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_info!("PANIC: S-mode panic!");
    loop {}
}
