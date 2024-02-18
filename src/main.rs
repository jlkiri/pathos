#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(abi_riscv_interrupt)]
#![feature(fn_ptr_trait)]
#![feature(const_mut_refs)]
#![feature(asm_const)]

mod asm;

use core::arch::asm;
use core::marker::FnPtr;
use core::panic::PanicInfo;
use hal::cpu::{Mideleg, Mstatus};
use lazy_static::lazy_static;

use pathos::interrupts::{self};
use pathos::{serial_debug, serial_error, serial_info};

#[no_mangle]
pub fn kinit() {
    serial_info!("Enter machine mode boot setup");

    let mideleg = Mideleg {
        sti: 1,
        ..Default::default()
    };

    let mstatus = Mstatus {
        mpp: 1,
        ..Default::default()
    };

    hal::cpu::write_mideleg(mideleg.clone());
    crate::serial_debug!("[WRITE] {}", mideleg);

    hal::cpu::write_mstatus(mstatus.clone());
    crate::serial_debug!("[WRITE] {}", mstatus);

    hal::cpu::write_mepc((main as fn()).addr());
    crate::serial_debug!("[WRITE] mepc <main> ::: {:?}", (main as fn()).addr());

    // interrupts::init_m_mode_ivt();

    // unsafe { asm!("mret") }
}

#[no_mangle]
pub fn main() {
    serial_info!("Enter supervisor mode boot setup");

    interrupts::init_s_mode_ivt();

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
    crate::serial_error!("Kernel panic!");

    let mstatus = hal::cpu::read_mstatus();
    let mie = hal::cpu::read_mie();
    let mip = hal::cpu::read_mip();
    let mcause = hal::cpu::read_mcause();
    let mtval = hal::cpu::read_mtval();
    let mepc = hal::cpu::read_mepc();

    crate::serial_debug!("{}", mstatus);
    crate::serial_debug!("{}", mie);
    crate::serial_debug!("{}", mip);
    crate::serial_debug!("mepc ::: {:?}", mepc);
    crate::serial_debug!("mtval ::: {:?}", mtval);
    crate::serial_debug!("M-mode: {}", mcause);

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
    let mtval = hal::cpu::read_mtval();
    let mepc = hal::cpu::read_mepc();

    crate::serial_debug!("{}", mstatus);
    crate::serial_debug!("{}", mie);
    crate::serial_debug!("{}", mip);
    crate::serial_debug!("mepc ::: {:?}", mepc);
    crate::serial_debug!("mtval ::: {:?}", mtval);
    crate::serial_debug!("M-mode: {}", mcause);

    loop {}
}
