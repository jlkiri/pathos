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
use hal::cpu::{Mideleg, Mstatus};
use lazy_static::lazy_static;

use pathos::interrupts::{self, InterruptIndex, InterruptVectorTable};
use pathos::{serial_debug, serial_error, serial_info};

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
pub fn kinit() {
    serial_info!("Enter machine mode boot setup");

    let mideleg = Mideleg {
        mti: 1,
        ..Default::default()
    };

    let mstatus = Mstatus {
        mpp: 1,
        ..Default::default()
    };

    hal::cpu::write_mideleg(mideleg);
    hal::cpu::write_mstatus(mstatus);
    // hal::cpu::write_mepc(main as *mut u8);
}

#[no_mangle]
pub fn main() {
    serial_info!("Enter supervisor mode boot setup");

    IVT.init();

    unsafe { asm!("li t0, 1 << 1", "csrs sstatus, t0") }

    serial_info!("Setup interrupt vector table");

    loop {}
}

#[no_mangle]
pub fn m_panic() -> ! {
    panic!()
}

#[cfg(test)]
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &PanicInfo) -> ! {
    let mstatus = hal::cpu::read_mstatus();
    let mie = hal::cpu::read_mie();
    let mip = hal::cpu::read_mip();
    let mcause = hal::cpu::read_mcause();

    crate::serial_debug!("{}", mstatus);
    crate::serial_debug!("{}", mie);
    crate::serial_debug!("{}", mip);
    crate::serial_debug!("{}", mcause);

    loop {}
}

#[cfg(not(test))]
#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    crate::serial_error!("Kernel panic!");

    let mstatus = hal::cpu::read_mstatus();
    let mie = hal::cpu::read_mie();
    let mip = hal::cpu::read_mip();
    let mcause = hal::cpu::read_mcause();

    crate::serial_debug!("{}", mstatus);
    crate::serial_debug!("{}", mie);
    crate::serial_debug!("{}", mip);
    crate::serial_debug!("{}", mcause);

    loop {}
}
