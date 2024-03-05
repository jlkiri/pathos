#![cfg_attr(target_os = "none", no_std)]
#![no_main]
#![feature(fn_align)]
#![feature(abi_riscv_interrupt)]
#![feature(fn_ptr_trait)]
#![feature(asm_const)]
#![feature(naked_functions)]

use core::{
    arch::asm,
    cell::OnceCell,
    mem::{offset_of, size_of},
    panic::PanicInfo,
    ptr,
};

use alloc::Locked;
use hal_core::page::Vaddr;
use once_cell::unsync::Lazy;

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

pub const APP_CODE: &[u8] = include_bytes!("app");

#[derive(Debug)]
pub struct Scheduler {
    tasks: [Task; 3],
    current: usize,
}

#[derive(Debug, Default)]
#[repr(align(8))]
pub struct TrapFrame {
    ra: u64,
    sp: u64,
    t0: u64,
    t1: u64,
    t2: u64,
    t3: u64,
    t4: u64,
    t5: u64,
    t6: u64,
    s0: u64,
    s1: u64,
    s2: u64,
    s3: u64,
    s4: u64,
    s5: u64,
    s6: u64,
    s7: u64,
    s8: u64,
    s9: u64,
    s10: u64,
    s11: u64,
    a0: u64,
    a1: u64,
    a2: u64,
    a3: u64,
    a4: u64,
    a5: u64,
    a6: u64,
    a7: u64,
    kernel_sp: usize,
}

#[derive(Debug)]
pub struct Task {
    pub trap_frame: TrapFrame,
    addr: Vaddr,
    pc: Vaddr,
}

impl Task {
    pub fn new(addr: Vaddr, tid: u64) -> Self {
        let kernel_sp = unsafe { KERNEL_STACK_END };
        let trap_frame = TrapFrame {
            kernel_sp,
            a0: tid as u64,
            ..TrapFrame::default()
        };
        Self {
            trap_frame,
            addr,
            pc: addr,
        }
    }
}

#[naked]
#[no_mangle]
pub unsafe fn save_cpu_registers() {
    asm!(
        "sd sp, 8(a0)",
        "sd t0, 16(a0)",
        "sd t1, 24(a0)",
        "sd t2, 32(a0)",
        "sd t3, 40(a0)",
        "sd t4, 48(a0)",
        "sd t5, 56(a0)",
        "sd t6, 64(a0)",
        "sd s0, 72(a0)",
        "sd s1, 80(a0)",
        "sd s2, 88(a0)",
        "sd s3, 96(a0)",
        "sd s4, 104(a0)",
        "sd s5, 112(a0)",
        "sd s6, 120(a0)",
        "sd s7, 128(a0)",
        "sd s8, 136(a0)",
        "sd s9, 144(a0)",
        "sd s10, 152(a0)",
        "sd s11, 160(a0)",
        "sd a1, 176(a0)",
        "sd a2, 184(a0)",
        "sd a3, 192(a0)",
        "sd a4, 200(a0)",
        "sd a5, 208(a0)",
        "sd a6, 216(a0)",
        "sd a7, 224(a0)",
        "csrr t0, mscratch",
        "sd t0, 168(a0)",
        "mv t0, zero",
        "ret",
        options(noreturn)
    )
}

#[naked]
#[no_mangle]
pub unsafe fn restore_cpu_registers() {
    asm!(
        "ld sp, 8(a0)",
        "ld t0, 16(a0)",
        "ld t1, 24(a0)",
        "ld t2, 32(a0)",
        "ld t3, 40(a0)",
        "ld t4, 48(a0)",
        "ld t5, 56(a0)",
        "ld t6, 64(a0)",
        "ld s0, 72(a0)",
        "ld s1, 80(a0)",
        "ld s2, 88(a0)",
        "ld s3, 96(a0)",
        "ld s4, 104(a0)",
        "ld s5, 112(a0)",
        "ld s6, 120(a0)",
        "ld s7, 128(a0)",
        "ld s8, 136(a0)",
        "ld s9, 144(a0)",
        "ld s10, 152(a0)",
        "ld s11, 160(a0)",
        "ld a1, 176(a0)",
        "ld a2, 184(a0)",
        "ld a3, 192(a0)",
        "ld a4, 200(a0)",
        "ld a5, 208(a0)",
        "ld a6, 216(a0)",
        "ld a7, 224(a0)",
        "ret",
        options(noreturn)
    )
}

pub static SCHEDULER: Locked<OnceCell<Scheduler>> = Locked::new(OnceCell::new());

pub fn init_scheduler(tasks: [Task; 3]) {
    let scheduler = SCHEDULER.lock();
    scheduler
        .set(Scheduler::new(tasks))
        .expect("Scheduler already initialized");
}

impl Scheduler {
    #[inline(always)]
    pub fn new(tasks: [Task; 3]) -> Self {
        Self { tasks, current: 0 }
    }

    pub fn current(&self) -> usize {
        self.current
    }

    #[inline(always)]
    pub fn task(&self, tid: usize) -> &Task {
        self.tasks.get(tid).expect("Invalid task index")
    }

    pub fn save_state(&mut self, addr: u64) {
        // let prev = (self.current + 2) % self.tasks.len();
        // let state = self.tasks.get_mut(prev).expect("Invalid task index");
        let state = self
            .tasks
            .get_mut(self.current)
            .expect("Invalid task index");
        state.pc = Vaddr::new(addr);
    }

    #[inline(always)]
    pub fn next(&mut self) -> (usize, &Task) {
        self.current = (self.current + 1) % self.tasks.len();
        (self.current, &self.tasks[self.current])
    }
}

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

pub fn nop_loop() {
    crate::serial_debug!("Entering NOP loop");
    loop {}
}
