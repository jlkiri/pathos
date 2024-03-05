extern crate alloc;

use crate::debug::{dump_machine_registers, dump_supervisor_registers, dump_trap_frame};
use crate::ecall::{self, ecall, Ecall};
use crate::serial::write_empty_line;
use crate::{
    nop_loop, restore_cpu_registers, save_cpu_registers, serial_debug, Scheduler, TrapFrame,
    SCHEDULER,
};
use core::marker::FnPtr;

use core::arch::asm;
use core::mem::size_of;
use core::panic;
use hal_core::page::Vaddr;
use hal_riscv::cpu::{self, read_mstatus, Exception, Interrupt, Mie, Mip, Mstatus, Sstatus};
use once_cell::unsync::Lazy;

#[inline(always)]
pub fn init_s_mode_ivt() {
    serial_debug!("[WRITE] stvec ::: {:?}", (stvec_table as fn()).addr());
    hal_riscv::cpu::write_stvec_vectored(stvec_table);
}

#[inline(always)]
pub fn init_m_mode_ivt() {
    // serial_debug!(
    //     "[WRITE] mtvec ::: {:?}",
    //     (interrupt_handler_naked as fn()).addr()
    // );
    // hal_riscv::cpu::write_mtvec_vectored(interrupt_handler_naked);
    hal_riscv::cpu::write_mtvec(interrupt_handler_naked);
}

// #[naked]
// fn handle_mti_naked() {
//     unsafe {
//         asm!(
//             "csrrw a0, mscratch, a0",
//             "sd ra, 0(a0)",
//             "jal {save_cpu_registers}",
//             "ld sp, 232(a0)",
//             "j {handle_mti}",
//             handle_mti = sym handle_mti,
//             save_cpu_registers = sym save_cpu_registers,
//             options(noreturn)
//         )
//     }
// }

#[inline(always)]
fn handle_mti() {
    // crate::serial_info!("Machine timer interrupt");

    let mut mtime = hal_riscv::timer::read_mtime();
    mtime += 10_000_000;
    hal_riscv::timer::write_mtimecmp(mtime);

    let mstatus = read_mstatus();
    let mstatus = Mstatus { mpp: 0, ..mstatus };
    cpu::write_mstatus(mstatus);

    schedule_next_program();
}

#[inline(always)]
fn schedule_next_program() {
    let (next_tid, mepc) = {
        let mut cell = SCHEDULER.lock();
        let scheduler = cell.get_mut().expect("Scheduler not initialized");

        let mepc = cpu::read_mepc() as u64;

        serial_debug!("Saving state for program ::: {:#x?}", mepc);

        scheduler.save_state(mepc);

        let (tid, task) = scheduler.next();

        cpu::write_mepc(task.pc.inner() as *const ());
        (tid, task.pc.inner())
    };

    let ptr = get_task_frame_ptr(next_tid);

    unsafe {
        asm!(
            "jal {restore_cpu_registers}",
            "csrw mscratch, a0",
            "ld ra, 0(a0)",
            "ld a0, 168(a0)",
            "mret",
            in("a0") ptr,
            restore_cpu_registers = sym restore_cpu_registers
        )
    }
}

extern "riscv-interrupt-s" fn noop() {
    crate::serial_debug!("[NOOP]");
}

#[no_mangle]
fn dispatch_machine_exception() {
    use hal_riscv::cpu::Cause;

    let mcause = cpu::read_mcause();
    match mcause {
        Cause::Exception(Exception::SupervisorEcall) => {
            let ecall = ecall::read_ecall();

            crate::serial_info!("S-mode ECALL ::: {:?}", ecall);

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
            schedule_next_program()
        }
        Cause::Exception(ref exc @ _) => {
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

#[no_mangle]
fn dispatch_supervisor_exception() {
    let scause = hal_riscv::cpu::read_scause();
    match scause {
        hal_riscv::cpu::Cause::Exception(Exception::UserEcall) => {
            let ecall = ecall::read_ecall();

            crate::serial_info!("U-mode ECALL ::: {:?}", ecall);

            match ecall {
                Ecall::Exit(_code) => {
                    dump_supervisor_registers();
                    // let sp = hal_riscv::cpu::read_sscratch();

                    crate::serial_error!("Program exited with code: {}", _code);
                    // crate::serial_error!("Restoring stack pointer from sscratch: {:x?}", sp);

                    // hal_riscv::cpu::write_sp(sp);
                    cpu::set_sstatus(Sstatus {
                        spp: 1,
                        ..Default::default()
                    });

                    hal_riscv::cpu::write_sepc(nop_loop as *const ());
                }
                _ => {
                    panic!("Unimplemented U-mode ecall handler ::: {:?}", scause)
                }
            }

            unsafe { asm!("sret", clobber_abi("system")) }
        }
        _ => {
            panic!("Unimplemented S-mode exception ::: {:?}", scause)
        }
    }
}

#[inline(never)]
fn get_task_frame_ptr(tid: usize) -> *const TrapFrame {
    let mut cell = SCHEDULER.lock();
    let scheduler = cell.get_mut().expect("Scheduler not initialized");
    &scheduler.task(tid).trap_frame as *const _
}

#[inline(always)]
fn handle_smode_finish_bootstrap() {
    serial_debug!("Finish supervisor bootstrap");

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
    hal_riscv::cpu::write_mepc(0x20_0000_0000 as *const ());

    crate::serial_debug!("[WRITE] {}", mie);
    crate::serial_debug!("[WRITE] {}", mstatus);

    let ptr = get_task_frame_ptr(0);

    unsafe {
        asm!(
            "jal {restore_cpu_registers}",
            "csrw mscratch, a0",
            "mv ra, zero",
            // "ld a0, 168(a0)",
            "mv a0, zero",
            "mret",
            in("a0") ptr,
            restore_cpu_registers = sym restore_cpu_registers
        )
    }
}

#[inline(always)]
#[repr(align(4))]
fn stvec_table() {
    unsafe {
        asm!(
            "jal {handle_exc}",
            ".org {stvec} + {ssi_idx} * 4",
            "jal {noop}",
            ".org {stvec} + {msi_idx} * 4",
            "jal {noop}",
            ".org {stvec} + {sti_idx} * 4",
            "jal {noop}",
            ".org {stvec} + {mti_idx} * 4",
            "jal {noop}",
            ".org {stvec} + {sei_idx} * 4",
            "jal {noop}",
            ".org {stvec} + {mei_idx} * 4",
            "jal {noop}",
            noop = sym noop,
            stvec = sym stvec_table,
            handle_exc = sym dispatch_supervisor_exception,
            ssi_idx = const Interrupt::SupervisorSoftware as u8,
            msi_idx = const Interrupt::MachineSoftware as u8,
            sti_idx = const Interrupt::SupervisorTimer as u8,
            mti_idx = const Interrupt::MachineTimer as u8,
            sei_idx = const Interrupt::SupervisorExternal as u8,
            mei_idx = const Interrupt::MachineExternal as u8,
            options(noreturn, nostack)
        )
    }
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
    use hal_riscv::cpu::Cause;
    let mcause = cpu::read_mcause();

    write_empty_line();
    serial_debug!("Machine cause: {:?}", mcause);

    if matches!(mcause, Cause::Exception(_)) {
        dispatch_machine_exception();
    }

    if matches!(mcause, Cause::Interrupt(Interrupt::MachineTimer)) {
        // unsafe { dump_trap_frame() }
        handle_mti();
    }

    unsafe { asm!("mret") }
}
