#![cfg_attr(target_os = "none", no_std)]
#![no_main]
#![feature(fn_align)]
#![feature(abi_riscv_interrupt)]
#![feature(fn_ptr_trait)]
#![feature(asm_const)]

use core::panic::PanicInfo;

pub mod alloc;
pub mod debug;
pub mod ecall;
pub mod interrupts;
pub mod page;
pub mod serial;

extern "C" {
    pub static TEXT_START: usize;
    pub static TEXT_END: usize;
    pub static DATA_START: usize;
    pub static DATA_END: usize;
    pub static BSS_START: usize;
    pub static BSS_END: usize;
    pub static KERNEL_STACK_START: usize;
    pub static KERNEL_STACK_END: usize;
    pub static HEAP_START: usize;
    pub static HEAP_SIZE: usize;
    pub static ALLOC_START: usize;
    pub static ALLOC_SIZE: usize;
    pub static MEMORY_START: usize;
    pub static MEMORY_END: usize;
    pub static RODATA_START: usize;
    pub static RODATA_END: usize;
}

// #[cfg(test)]
// #[panic_handler]
// #[no_mangle]
// pub fn panic(info: &PanicInfo) -> ! {
//     use debug::{dump_machine_registers, dump_supervisor_registers};

//     crate::serial_error!(" ");
//     crate::serial_error!("*** KERNEL PANIC ***");
//     crate::serial_error!(" ");
//     crate::serial_error!("{}", info);

//     dump_machine_registers();
//     dump_supervisor_registers();

//     loop {}
// }

#[cfg(all(not(test), target_os = "none"))]
#[panic_handler]
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    use debug::{dump_machine_registers, dump_supervisor_registers};

    crate::serial_error!(" ");
    crate::serial_error!("*** KERNEL PANIC ***");
    crate::serial_error!(" ");
    crate::serial_error!("{}", info);

    dump_machine_registers();
    dump_supervisor_registers();

    loop {}
}
