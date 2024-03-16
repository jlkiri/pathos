extern crate alloc;

use crate::debug::{dump_machine_registers, dump_supervisor_registers};
use crate::ecall::{self, ecall, Ecall};
use crate::{nop_loop, serial_debug};
use core::marker::FnPtr;

use core::arch::asm;
use core::panic;
use hal_riscv::cpu::{self, Exception, Interrupt, Mie, Mip, Mstatus, Sstatus};

#[inline(always)]
pub fn init_s_mode_ivt() {
    serial_debug!("[WRITE] stvec ::: {:?}", (stvec_table as fn()).addr());
    hal_riscv::cpu::write_stvec_vectored(stvec_table);
    // ecall(Ecall::SModeFinishBootstrap)
}

#[inline(always)]
pub fn init_m_mode_ivt() {
    serial_debug!("[WRITE] mtvec ::: {:?}", (mtvec_table as fn()).addr());
    hal_riscv::cpu::write_mtvec_vectored(mtvec_table);
}

extern "riscv-interrupt-s" fn handle_sti() {
    crate::serial_info!("Software timer interrupt");

    dump_supervisor_registers();

    ecall(Ecall::ClearPendingInterrupt(
        Interrupt::SupervisorTimer as u8,
    ));
}

extern "riscv-interrupt-m" fn handle_mti() {
    crate::serial_info!("Machine timer interrupt");

    let mut mtime = hal_riscv::timer::read_mtime();
    mtime += 20_000_000;
    hal_riscv::timer::write_mtimecmp(mtime);

    // let mip = hal_riscv::cpu::read_mip();

    dump_machine_registers();

    // let mip = Mip { stip: 1, ..mip };
    // hal_riscv::cpu::write_mip(mip.clone());
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
                Ecall::ClearPendingInterrupt(cause) => handle_clear_pending_interrupt(cause),
                _ => {
                    panic!("Unimplemented S-mode ecall handler ::: {:?}", mcause)
                }
            }
        }
        Cause::Exception(Exception::InstructionPageFault) => {
            dump_machine_registers();
            serial_debug!("Instruction page fault ::: {:?}", mcause);
            loop {}
        }
        Cause::Exception(Exception::InstructionFault) => {
            dump_machine_registers();
            serial_debug!("Instruction access fault ::: {:?}", mcause);
            loop {}
        }
        Cause::Exception(Exception::IllegalInstruction) => {
            dump_machine_registers();
            serial_debug!("Illegal instruction ::: {:?}", mcause);
            loop {}
        }
        Cause::Exception(Exception::Breakpoint) => {
            dump_machine_registers();
            serial_debug!("Breakpoint ::: {:?}", mcause);
            loop {}
        }
        Cause::Exception(Exception::UserEcall) => {
            dump_machine_registers();
            serial_debug!("User ecall ::: {:?}", mcause);
            let mstatus = cpu::read_mstatus();
            let mstatus = Mstatus { mpp: 1, ..mstatus };
            cpu::write_mstatus(mstatus);
            cpu::write_mepc((nop_loop as fn()).addr());
        }
        _ => {
            // dump_machine_registers();
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
                    let sp = hal_riscv::cpu::read_sscratch();
                    hal_riscv::cpu::write_sp(sp);

                    crate::serial_error!("Restored stack pointer: {:x?}", sp);
                    crate::serial_error!("Program exited with code: {}", _code);

                    let sstatus = Sstatus {
                        spp: 1,
                        ..Default::default()
                    };
                    cpu::set_sstatus(sstatus);

                    hal_riscv::cpu::write_sepc((nop_loop as fn()).addr());
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

#[inline(always)]
fn handle_smode_finish_bootstrap() {
    let mie = Mie {
        // mtie: 1,
        // stie: 1,
        ..Default::default()
    };

    let mstatus = hal_riscv::cpu::read_mstatus();
    let mstatus = Mstatus {
        mpp: 1,
        // mpie: 1,
        fs: 1,
        ..mstatus
    };

    hal_riscv::cpu::write_mie(mie.clone());
    hal_riscv::cpu::write_mstatus(mstatus.clone());
    hal_riscv::cpu::write_mepc_next();

    crate::serial_debug!("[WRITE] {}", mie);
    crate::serial_debug!("[WRITE] {}", mstatus);
}

#[inline(always)]
fn handle_clear_pending_interrupt(cause: u8) {
    hal_riscv::cpu::clear_mip(cause);
    hal_riscv::cpu::write_mepc_next();
}

#[no_mangle]
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
            "jal {handle_sti}",
            ".org {stvec} + {mti_idx} * 4",
            "jal {noop}",
            ".org {stvec} + {sei_idx} * 4",
            "jal {noop}",
            ".org {stvec} + {mei_idx} * 4",
            "jal {noop}",
            noop = sym noop,
            stvec = sym stvec_table,
            handle_exc = sym dispatch_supervisor_exception,
            handle_sti = sym handle_sti,
            ssi_idx = const Interrupt::SupervisorSoftware as u8,
            msi_idx = const Interrupt::MachineSoftware as u8,
            sti_idx = const Interrupt::SupervisorTimer as u8,
            mti_idx = const Interrupt::MachineTimer as u8,
            sei_idx = const Interrupt::SupervisorExternal as u8,
            mei_idx = const Interrupt::MachineExternal as u8,
            options(noreturn)
        )
    }
}

#[no_mangle]
#[repr(align(4))]
fn mtvec_table() {
    unsafe {
        asm!(
            "jal {handle_exc}",
            ".org {mtvec} + {ssi_idx} * 4",
            "jal {noop}",
            ".org {mtvec} + {msi_idx} * 4",
            "jal {noop}",
            ".org {mtvec} + {sti_idx} * 4",
            "jal {noop}",
            ".org {mtvec} + {mti_idx} * 4",
            "jal {handle_mti}",
            ".org {mtvec} + {sei_idx} * 4",
            "jal {noop}",
            ".org {mtvec} + {mei_idx} * 4",
            "jal {noop}",
            noop = sym noop,
            mtvec = sym mtvec_table,
            handle_exc = sym dispatch_machine_exception,
            handle_mti = sym handle_mti,
            ssi_idx = const Interrupt::SupervisorSoftware as u8,
            msi_idx = const Interrupt::MachineSoftware as u8,
            sti_idx = const Interrupt::SupervisorTimer as u8,
            mti_idx = const Interrupt::MachineTimer as u8,
            sei_idx = const Interrupt::SupervisorExternal as u8,
            mei_idx = const Interrupt::MachineExternal as u8,
            options(noreturn)
        )
    }
}
