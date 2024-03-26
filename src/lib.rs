#![cfg_attr(target_os = "none", no_std)]
#![no_main]
#![feature(fn_align)]
#![feature(abi_riscv_interrupt)]
#![feature(fn_ptr_trait)]
#![feature(asm_const)]
#![feature(naked_functions)]

use core::panic::PanicInfo;

use hal_core::page::{EntryFlags, Page, PageTable, Vaddr};
use page::*;
use trap::{Scheduler, Task, SCHEDULER};

pub mod alloc;
pub mod constants;
pub mod debug;
pub mod ecall;
pub mod elf;
pub mod interrupts;
pub mod page;
pub mod serial;
pub mod trap;

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

pub const APP_CODE: &[u8] = include_bytes!("app");

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

pub unsafe fn init_page_tables(root: &mut PageTable) {
    // I needed to set .text and .rodata to X because otherwise I
    // get store page faults on AMO instructions in the kernel
    // (which probably belong to mutexes or spinlocks).

    id_map_range(root, TEXT_START, TEXT_END, EntryFlags::RWX);
    serial_debug!(
        "Identity mapped kernel .text: 0x{:x} - 0x{:x}",
        TEXT_START,
        TEXT_END
    );

    id_map_range(root, RODATA_START, RODATA_END, EntryFlags::RWX);
    serial_debug!(
        "Identity mapped kernel .rodata: 0x{:x} - 0x{:x}",
        RODATA_START,
        RODATA_END
    );

    id_map_range(root, DATA_START, DATA_END, EntryFlags::RW);
    serial_debug!(
        "Identity mapped kernel .data: 0x{:x} - 0x{:x}",
        DATA_START,
        DATA_END
    );

    id_map_range(root, BSS_START, BSS_END, EntryFlags::RW);
    serial_debug!(
        "Identity mapped kernel .bss: 0x{:x} - 0x{:x}",
        BSS_START,
        BSS_END
    );

    id_map_range(root, KERNEL_STACK_START, KERNEL_STACK_END, EntryFlags::RW);
    serial_debug!(
        "Identity mapped kernel stack: 0x{:x} - 0x{:x}",
        KERNEL_STACK_START,
        KERNEL_STACK_END
    );

    id_map_range(root, HEAP_START, HEAP_START + HEAP_SIZE, EntryFlags::RW);
    serial_debug!(
        "Identity mapped kernel heap: 0x{:x} - 0x{:x}",
        HEAP_START,
        HEAP_START + HEAP_SIZE
    );

    id_map_range(root, ALLOC_START, ALLOC_START + ALLOC_SIZE, EntryFlags::RW);
    serial_debug!(
        "Identity mapped kernel allocatable memory: 0x{:x} - 0x{:x}",
        ALLOC_START,
        ALLOC_START + ALLOC_SIZE
    );

    id_map(root, Page::containing_address(0x10000000), EntryFlags::RW);
    serial_debug!("Identity mapped UART device: 0x{:x}", 0x10000000);

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
        if translate_vaddr(root, vaddr).is_none() {
            panic!("0x{:x} cannot be translated", vaddr.inner());
        }
    }

    // TODO: Check why not every address translation in HEAP and ALLOCATE
    // sections works.
}

pub fn init_scheduler(tasks: [Task; 3]) {
    let scheduler = SCHEDULER.lock();
    scheduler
        .set(Scheduler::new(tasks))
        .expect("Scheduler already initialized");
}
