#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(abi_riscv_interrupt)]
#![feature(fn_ptr_trait)]
#![feature(asm_const)]

mod asm;

extern crate alloc;

use ::core::arch::asm;
use ::core::marker::FnPtr;
use alloc::vec::Vec;
use hal_core::page::{EntryFlags, Frame, Page, PageTable, Vaddr};
use hal_riscv::cpu::{Mideleg, Mstatus, Satp, Sstatus};
use pathos::alloc::init_allocator;
use pathos::constants::TASK_BEGIN_VADDR;
use pathos::ecall::{ecall, Ecall};
use pathos::elf::parse_text;
use pathos::trap::Task;
use pathos::{init_page_tables, init_scheduler, interrupts, page, APP_CODE};
use pathos::{serial_debug, serial_info, serial_println};

const LOGO: &str = include_str!("logo.txt");

#[no_mangle]
pub fn kinit() {
    serial_println!("{}", LOGO);

    let mideleg = Mideleg {
        ..Default::default()
    };

    // let medeleg = Medeleg { uecall: 1 };
    let mstatus = Mstatus {
        mpp: 1,
        fs: 1,
        ..Default::default()
    };

    hal_riscv::cpu::write_mideleg(mideleg.clone());
    hal_riscv::cpu::write_mstatus(mstatus.clone());
    hal_riscv::cpu::write_mepc((main as fn()).addr());

    init_scheduler([
        Task::new(Vaddr::new(0x20_0000_0000), 0),
        Task::new(Vaddr::new(0x20_0000_0000), 1),
        Task::new(Vaddr::new(0x20_0000_0000), 2),
    ]);

    serial_info!("Initialized task scheduler");

    interrupts::init_m_mode_ivt();
    serial_info!("Initialized machine mode interrupt vector table");

    unsafe { asm!("mret") }
}

#[no_mangle]
pub fn main() {
    init_allocator();
    serial_info!("Initialized global heap allocator");

    // Identity map kernel code and data before switching to Sv39 paging
    let root = page::allocate_root();
    unsafe {
        init_page_tables(root);
    }

    map_userspace_program(root);

    // Create satp entry and enable Sv39 paging
    let satp = Satp::new(8, root as *mut PageTable as usize);
    hal_riscv::cpu::write_satp(satp);

    serial_info!("Enabled Sv39 paging");

    let sstatus = Sstatus {
        spp: 0,
        ..Default::default()
    };

    hal_riscv::cpu::set_sstatus(sstatus);
    ecall(Ecall::SModeFinishBootstrap);

    loop {
        unsafe { asm!("wfi") }
    }
}

fn map_userspace_program(root: &mut PageTable) {
    // Parse userspace binary text section and put data somewhere in the heap
    let program_text_section = parse_text(APP_CODE);
    let data = Vec::from(program_text_section).leak();
    serial_debug!("Copied user program to address: {:#x?}", data.as_ptr());

    let pstart = data.as_ptr() as usize;

    page::map_range(
        root,
        TASK_BEGIN_VADDR as usize,
        pstart,
        1024 * 1024,
        EntryFlags::RWXU,
    );

    // Expose UART MMIO
    page::map(
        root,
        Page::containing_address(0x10_0000_0000),
        Frame::containing_address(0x10000000),
        EntryFlags::RWU,
    );

    unsafe { asm!("sfence.vma zero, zero") }

    serial_debug!(
        "Mapped user space memory: {:#x?} - {:#x?}",
        TASK_BEGIN_VADDR,
        TASK_BEGIN_VADDR + 1024 * 1024
    );
}
