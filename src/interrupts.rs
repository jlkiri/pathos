use core::arch::asm;

#[repr(u8)]
enum InterruptIndex {
    SupervisorSoftware = 1,
    MachineSoftware = 3,
    SupervisorTimer = 5,
    MachineTimer = 7,
    SupervisorExternal = 9,
    MachineExternal = 11,
}

pub struct MachineInterruptVectorTable;

#[inline(always)]
pub fn init_s_mode_ivt() {
    hal::cpu::write_stvec_vectored(stvec_table);
    unsafe { asm!("li x31, 1", "ecall") }
}

#[inline(always)]
pub fn init_m_mode_ivt() {
    hal::cpu::write_mtvec(mtvec_table);
}

extern "riscv-interrupt-s" fn handle_sti() {
    crate::serial_info!("Handle pending software timer interrupt");

    let sstatus = hal::cpu::read_sstatus();
    let sie = hal::cpu::read_sie();
    let sip = hal::cpu::read_sip();
    let scause = hal::cpu::read_scause();

    crate::serial_debug!("{}", sstatus);
    crate::serial_debug!("{}", sie);
    crate::serial_debug!("{}", sip);
    crate::serial_debug!("{}", scause);

    unsafe { asm!("li x31, 2", "ecall") }
}

extern "riscv-interrupt-m" fn handle_mti() {
    crate::serial_info!("Handle pending machine timer interrupt");

    let mstatus = hal::cpu::read_mstatus();
    let mie = hal::cpu::read_mie();
    let mip = hal::cpu::read_mip();
    let mcause = hal::cpu::read_mcause();

    crate::serial_debug!("{}", mstatus);
    crate::serial_debug!("{}", mie);
    crate::serial_debug!("{}", mip);
    crate::serial_debug!("{}", mcause);
}

extern "riscv-interrupt-s" fn noop() {
    crate::serial_info!("NOOP");
    unsafe { asm!("li x31, 2", "ecall") }
}

fn handle_machine_exception() {
    let mcause = hal::cpu::read_mcause();
    match mcause {
        hal::cpu::Cause::Exception(9) => {}
        _ => panic!(),
    }

    crate::serial_info!("Handle machine exception");
    // panic!()
    // loop {}
}

#[no_mangle]
#[repr(align(4))]
fn stvec_table() {
    unsafe {
        asm!(
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
            handle_sti = sym handle_sti,
            ssi_idx = const InterruptIndex::SupervisorSoftware as u8,
            msi_idx = const InterruptIndex::MachineSoftware as u8,
            sti_idx = const InterruptIndex::SupervisorTimer as u8,
            mti_idx = const InterruptIndex::MachineTimer as u8,
            sei_idx = const InterruptIndex::SupervisorExternal as u8,
            mei_idx = const InterruptIndex::MachineExternal as u8,
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
            handle_exc = sym handle_machine_exception,
            handle_mti = sym handle_mti,
            ssi_idx = const InterruptIndex::SupervisorSoftware as u8,
            msi_idx = const InterruptIndex::MachineSoftware as u8,
            sti_idx = const InterruptIndex::SupervisorTimer as u8,
            mti_idx = const InterruptIndex::MachineTimer as u8,
            sei_idx = const InterruptIndex::SupervisorExternal as u8,
            mei_idx = const InterruptIndex::MachineExternal as u8,
            options(noreturn)
        )
    }
}
