extern crate alloc;

use crate::constants::TASK_BEGIN_VADDR;
use crate::debug::dump_machine_registers;
use crate::ecall::{self, Ecall};
use crate::serial::write_empty_line;
use crate::trap::{restore_cpu_registers, save_cpu_registers, TrapFrame};
use crate::{serial_debug, SCHEDULER};

use core::arch::asm;
use core::panic;
use hal_riscv::cpu::{self, read_mstatus, Cause, Exception, Interrupt, Mie, Mstatus};

#[inline(always)]
pub fn init_m_mode_ivt() {
    hal_riscv::cpu::write_mtvec(interrupt_handler_naked);
}

#[inline(always)]
fn handle_mti() {
    let mut mtime = hal_riscv::timer::read_mtime();
    mtime += 10_000_000;
    hal_riscv::timer::write_mtimecmp(mtime);

    let mstatus = read_mstatus();
    let mstatus = Mstatus { mpp: 0, ..mstatus };
    cpu::write_mstatus(mstatus);

    let mepc = cpu::read_mepc() as u64;

    schedule_task(UserspaceState::Running(mepc));
}

enum UserspaceState {
    Running(u64),
    Pending,
}

#[inline(always)]
fn schedule_task(state: UserspaceState) {
    let (next_tid, next_mepc) = match state {
        UserspaceState::Pending => (0, TASK_BEGIN_VADDR),
        UserspaceState::Running(mepc) => {
            let mut cell = SCHEDULER.lock();
            let scheduler = cell.get_mut().expect("Scheduler not initialized");
            scheduler.save_state(mepc);
            let (tid, task) = scheduler.next();
            (tid, task.pc.inner())
        }
    };

    cpu::write_mepc(next_mepc as *const ());

    unsafe {
        asm!(
            "jal {restore_cpu_registers}",
            "csrw mscratch, a0",
            "ld ra, 0(a0)",
            "ld a0, 168(a0)",
            "mret",
            in("a0") get_task_frame_ptr(next_tid),
            restore_cpu_registers = sym restore_cpu_registers
        )
    }
}

#[no_mangle]
fn dispatch_machine_exception(mcause: Cause) {
    match mcause {
        Cause::Exception(Exception::SupervisorEcall) => {
            // dump_machine_registers();
            let ecall = ecall::read_ecall();
            match ecall {
                Ecall::SModeFinishBootstrap => handle_smode_finish_bootstrap(),
                _ => {
                    panic!("Unimplemented S-mode ecall handler ::: {:?}", mcause)
                }
            }
        }
        Cause::Exception(Exception::UserEcall) => {
            dump_machine_registers();
            serial_debug!("{:?} ::: {:?}", Exception::UserEcall, mcause);
            schedule_task(UserspaceState::Pending)
        }
        Cause::Exception(ref exc) => {
            dump_machine_registers();
            serial_debug!("{:?} ::: {:?}", exc, mcause);
            loop {}
        }
        _ => {
            dump_machine_registers();
            panic!("Unimplemented M-mode exception ::: {:?}", mcause)
        }
    }

    unsafe { asm!("mret", clobber_abi("system")) }
}

#[inline(never)]
fn get_task_frame_ptr(tid: usize) -> *const TrapFrame {
    let mut cell = SCHEDULER.lock();
    let scheduler = cell.get_mut().expect("Scheduler not initialized");
    &scheduler.task(tid).trap_frame as *const _
}

#[inline(always)]
fn handle_smode_finish_bootstrap() {
    let mie = Mie {
        mtie: 1,
        ..Default::default()
    };

    let mstatus = hal_riscv::cpu::read_mstatus();
    let mstatus = Mstatus {
        mpp: 0,
        mpie: 1,
        fs: 1,
        ..mstatus
    };

    hal_riscv::cpu::write_mie(mie.clone());
    hal_riscv::cpu::write_mstatus(mstatus.clone());
    schedule_task(UserspaceState::Pending)
}

#[inline(always)]
#[no_mangle]
#[repr(align(4))]
fn interrupt_handler_naked() {
    unsafe {
        asm!(
            "csrrw a0, mscratch, a0",
            "beqz a0, {interrupt_handler}",
            "sd ra, 0(a0)",
            "jal {save_cpu_registers}",
            "ld sp, 232(a0)",
            "j {interrupt_handler}",
            save_cpu_registers = sym save_cpu_registers,
            interrupt_handler = sym interrupt_handler,
            options(noreturn)
        )
    }
}

#[inline(always)]
fn interrupt_handler() {
    let mcause = cpu::read_mcause();

    if matches!(mcause, Cause::Exception(_)) {
        serial_debug!("Machine mode exception cause: {:?}", mcause);
        dispatch_machine_exception(mcause.clone());
    }

    if matches!(mcause, Cause::Interrupt(Interrupt::MachineTimer)) {
        write_empty_line();
        // serial_debug!("Machine mode interrupt cause: {:?}", mcause);
        // unsafe { dump_trap_frame() }
        handle_mti();
    }

    unsafe { asm!("mret") }
}
