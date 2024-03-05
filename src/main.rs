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
use hal_core::page::{EntryFlags, Page, PageTable, Vaddr};
use hal_riscv::cpu::{Mideleg, Mstatus, Satp, Sstatus};
use pathos::alloc::init_allocator;
use pathos::{
    interrupts, page, ALLOC_SIZE, ALLOC_START, BSS_END, BSS_START, DATA_END, DATA_START, HEAP_SIZE,
    HEAP_START, KERNEL_STACK_END, KERNEL_STACK_START, RODATA_END, RODATA_START, TEXT_END,
    TEXT_START,
};
use pathos::{serial_debug, serial_info, serial_println};
// use wasmi::*;

const LOGO: &str = include_str!("logo.txt");

#[no_mangle]
pub fn kinit() {
    serial_println!("{}", LOGO);

    let mideleg = Mideleg {
        sti: 1,
        ..Default::default()
    };

    let mstatus = Mstatus {
        mpp: 1,
        ..Default::default()
    };

    hal_riscv::cpu::write_mideleg(mideleg.clone());
    crate::serial_debug!("[WRITE] {}", mideleg);

    hal_riscv::cpu::write_mstatus(mstatus.clone());
    crate::serial_debug!("[WRITE] {}", mstatus);

    hal_riscv::cpu::write_mepc((main as fn()).addr());
    crate::serial_debug!("[WRITE] mepc <main> ::: {:?}", (main as fn()).addr());

    interrupts::init_m_mode_ivt();
    serial_debug!("Initialized M-mode interrupt vector table");

    unsafe { asm!("mret") }
}

#[no_mangle]
pub fn main() {
    init_allocator();
    serial_debug!("Initialized global heap allocator");

    // Identity map kernel code and data before switching to Sv39 paging
    let root = page::allocate_root();
    unsafe { init_page_tables(root) }

    unsafe {
        // Now perform sanity check by trying to translate each section's start
        // virtual address to a physical address.
        let vaddr = Vaddr::new(TEXT_START as u64);
        if page::translate_vaddr(root, vaddr).is_none() {
            panic!("0x{:x} cannot be translated", vaddr.inner());
        }

        let vaddr = Vaddr::new(DATA_START as u64);
        if page::translate_vaddr(root, vaddr).is_none() {
            panic!("0x{:x} cannot be translated", vaddr.inner());
        }

        let vaddr = Vaddr::new(BSS_START as u64);
        if page::translate_vaddr(root, vaddr).is_none() {
            panic!("0x{:x} cannot be translated", vaddr.inner());
        }

        let vaddr = Vaddr::new(KERNEL_STACK_START as u64);
        if page::translate_vaddr(root, vaddr).is_none() {
            panic!("0x{:x} cannot be translated", vaddr.inner());
        }

        // TODO: Check why not every address translation in HEAP and ALLOCATE
        // sections works.
    }

    // Create satp entry and enable Sv39 paging
    let satp = Satp::new(8, root as *mut PageTable as usize);

    serial_debug!("[WRITE] satp ::: {}", satp);
    hal_riscv::cpu::write_satp(satp.clone());

    serial_info!("Enabled Sv39 paging");

    interrupts::init_s_mode_ivt();
    serial_debug!("Initialized S-mode interrupt vector table");

    let sstatus = hal_riscv::cpu::read_sstatus();
    let sstatus = Sstatus { sie: 1, ..sstatus };
    hal_riscv::cpu::set_sstatus(sstatus);

    // let engine = Engine::default();
    // let wat = r#"
    //     (module
    //         (import "host" "hello" (func $host_hello (param i32)))
    //         (func (export "hello")
    //             (call $host_hello (i32.const 3))
    //         )
    //     )
    // "#;

    // let wasm = include_bytes!("../test.wasm").to_vec();
    // let module = Module::new(&engine, &wasm[..]).expect("Failed to create module");
    // type HostState = u32;
    // let mut store = Store::new(&engine, 42);
    // let host_hello = Func::wrap(&mut store, |caller: Caller<'_, HostState>, param: i32| {
    //     // println!("Got {param} from WebAssembly");
    //     // println!("My host state is: {}", caller.data());
    // });
    // let mut linker = <Linker<HostState>>::new(&engine);
    // linker
    //     .define("host", "hello", host_hello)
    //     .expect("Failed to define host function");
    // let instance = linker
    //     .instantiate(&mut store, &module)
    //     .expect("Failed to instantiate module")
    //     .start(&mut store)
    //     .expect("Failed to start module");
    // let hello = instance
    //     .get_typed_func::<(), ()>(&store, "hello")
    //     .expect("Failed to get function");
    // // And finally we can call the wasm!
    // hello.call(&mut store, ()).expect("Failed to call function");

    loop {}
}

unsafe fn init_page_tables(root: &mut PageTable) {
    page::id_map_range(root, TEXT_START, TEXT_END, EntryFlags::RX);
    serial_debug!(
        "Identity mapped kernel .text: 0x{:x} - 0x{:x}",
        TEXT_START,
        TEXT_END
    );

    page::id_map_range(root, RODATA_START, RODATA_END, EntryFlags::RX);
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
