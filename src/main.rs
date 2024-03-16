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
use alloc::boxed::Box;
use alloc::vec::{self, Vec};
use core::ops::Add;
use core::ptr;
use elf::endian::LittleEndian;
use elf::section::SectionHeader;
use elf::ElfBytes;
use hal_core::page::{EntryFlags, Page, PageTable, Vaddr};
use hal_riscv::cpu::{Medeleg, Mideleg, Mstatus, Satp, Sstatus};
use pathos::alloc::init_allocator;
use pathos::debug::dump_supervisor_registers;
use pathos::{
    interrupts, nop_loop, page, serial_print, ALLOC_SIZE, ALLOC_START, APP_CODE, BSS_END,
    BSS_START, DATA_END, DATA_START, HEAP_SIZE, HEAP_START, KERNEL_STACK_END, KERNEL_STACK_START,
    RODATA_END, RODATA_START, TEXT_END, TEXT_START,
};
use pathos::{serial_debug, serial_info, serial_println};

const LOGO: &str = include_str!("logo.txt");

#[no_mangle]
pub fn kinit() {
    serial_println!("{}", LOGO);

    let mideleg = Mideleg {
        sti: 1,
        ..Default::default()
    };

    let medeleg = Medeleg { uecall: 1 };
    let mstatus = Mstatus {
        mpp: 1,
        ..Default::default()
    };

    hal_riscv::cpu::write_mideleg(mideleg.clone());
    crate::serial_debug!("[WRITE] {}", mideleg);

    hal_riscv::cpu::write_medeleg(medeleg.clone());
    crate::serial_debug!("[WRITE] {}", medeleg);

    hal_riscv::cpu::write_mstatus(mstatus.clone());
    crate::serial_debug!("[WRITE] {}", mstatus);

    hal_riscv::cpu::write_mepc((main as fn()).addr());
    crate::serial_debug!("[WRITE] mepc <main> ::: {:?}", (main as fn()).addr());

    interrupts::init_m_mode_ivt();
    serial_debug!("Initialized M-mode interrupt vector table");

    unsafe { asm!("mret") }

    loop {
        unsafe { asm!("wfi") }
    }
}

#[no_mangle]
pub fn main() {
    init_allocator();
    serial_debug!("Initialized global heap allocator");

    // Identity map kernel code and data before switching to Sv39 paging
    let root = page::allocate_root();

    unsafe {
        init_page_tables(root);

        // Now perform sanity check by trying to translate each section's start
        // and end virtual addresses to a physical address.
        for vaddr in [
            TEXT_START,
            TEXT_END,
            RODATA_START,
            RODATA_END,
            DATA_START,
            DATA_END,
            BSS_START,
            BSS_END,
            KERNEL_STACK_START,
            KERNEL_STACK_END,
            // HEAP_START,
            // HEAP_START + HEAP_SIZE,
            // ALLOC_START,
            // ALLOC_START + ALLOC_SIZE,
        ] {
            let vaddr = Vaddr::new(vaddr as u64);
            if page::translate_vaddr(root, vaddr).is_none() {
                panic!("0x{:x} cannot be translated", vaddr.inner());
            }
        }

        // TODO: Check why not every address translation in HEAP and ALLOCATE
        // sections works.
    }

    serial_debug!("Sanity check passed");

    let file = ElfBytes::<LittleEndian>::minimal_parse(APP_CODE).expect("Failed to parse ELF file");

    let text_section: SectionHeader = file
        .section_header_by_name(".text")
        .expect("Failed to find .text section")
        .expect("Failed to parse .text section");

    let data = file
        .section_data(&text_section)
        .expect("Failed to read .text section");

    // Put data somewhere in the heap
    let data = Vec::from(data.0).leak();
    serial_debug!("Copied user program to address: {:#x?}", data.as_ptr());

    let vstart = 0x20_0000_0000;
    let pstart = data.as_ptr() as usize;

    page::map_range(root, vstart, pstart, 4096, EntryFlags::RWXU);
    unsafe { asm!("sfence.vma zero, zero") }

    serial_debug!(
        "Mapped user space memory: {:#x?} - {:#x?}",
        vstart,
        vstart + 4096
    );

    // Create satp entry and enable Sv39 paging
    let satp = Satp::new(8, root as *mut PageTable as usize);

    serial_debug!("[WRITE] satp ::: {}", satp);
    hal_riscv::cpu::write_satp(satp.clone());

    serial_info!("Enabled Sv39 paging");

    // interrupts::init_s_mode_ivt();
    // serial_debug!("Initialized S-mode interrupt vector table");

    // let sstatus = hal_riscv::cpu::read_sstatus();
    let sstatus = Sstatus {
        // sie: 1,
        spp: 0,
        ..Default::default()
    };

    // Allocate enough virtual space for the program starting at 0x20_0000_0000,
    // map the address range & mark it as executable. After that,
    // load the program into the allocated memory.

    // let src = Vaddr::new(data.0.as_ptr() as u64);
    // let dst = Vaddr::new(0x20_0000_0000 as u64);

    // serial_debug!(
    //     "src: {:x?}, dst: {:x?}",
    //     src.inner() as *const u8,
    //     dst.inner() as *mut u8
    // );

    // unsafe {
    //     ptr::copy_nonoverlapping(
    //         src.inner() as *const u8,
    //         dst.inner() as *mut u8,
    //         data.0.len(),
    //     );
    //     serial_debug!("Loaded program into 0x20_0000_0000");
    // }

    let sp = hal_riscv::cpu::read_sp();
    hal_riscv::cpu::write_sscratch(sp);

    serial_debug!("Saved stack pointer to sscratch: 0x{:x}", sp);

    hal_riscv::cpu::write_sepc(0x20_0000_0000 as *const ());
    hal_riscv::cpu::set_sstatus(sstatus);

    unsafe { asm!("sret") }

    loop {
        unsafe { asm!("wfi") }
    }
}

unsafe fn init_page_tables(root: &mut PageTable) {
    // I needed to set .text and .rodata to X because otherwise I
    // get store page faults on AMO instructions in the kernel
    // (which probably belong to mutexes or spinlocks).

    page::id_map_range(root, TEXT_START, TEXT_END, EntryFlags::RWX);
    serial_debug!(
        "Identity mapped kernel .text: 0x{:x} - 0x{:x}",
        TEXT_START,
        TEXT_END
    );

    page::id_map_range(root, RODATA_START, RODATA_END, EntryFlags::RWX);
    serial_debug!(
        "Identity mapped kernel .rodata: 0x{:x} - 0x{:x}",
        RODATA_START,
        RODATA_END
    );

    page::id_map_range(root, DATA_START, DATA_END, EntryFlags::RW);
    serial_debug!(
        "Identity mapped kernel .data: 0x{:x} - 0x{:x}",
        DATA_START,
        DATA_END
    );

    page::id_map_range(root, BSS_START, BSS_END, EntryFlags::RW);
    serial_debug!(
        "Identity mapped kernel .bss: 0x{:x} - 0x{:x}",
        BSS_START,
        BSS_END
    );

    page::id_map_range(root, KERNEL_STACK_START, KERNEL_STACK_END, EntryFlags::RW);
    serial_debug!(
        "Identity mapped kernel stack: 0x{:x} - 0x{:x}",
        KERNEL_STACK_START,
        KERNEL_STACK_END
    );

    page::id_map_range(root, HEAP_START, HEAP_START + HEAP_SIZE, EntryFlags::RW);
    serial_debug!(
        "Identity mapped kernel heap: 0x{:x} - 0x{:x}",
        HEAP_START,
        HEAP_START + HEAP_SIZE
    );

    page::id_map_range(root, ALLOC_START, ALLOC_START + ALLOC_SIZE, EntryFlags::RW);
    serial_debug!(
        "Identity mapped kernel allocatable memory: 0x{:x} - 0x{:x}",
        ALLOC_START,
        ALLOC_START + ALLOC_SIZE
    );

    page::id_map(root, Page::containing_address(0x10000000), EntryFlags::RW);
    serial_debug!("Identity mapped UART device: 0x{:x}", 0x10000000);
}
