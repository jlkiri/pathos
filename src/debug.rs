#[inline(always)]
pub fn dump_machine_registers() {
    let mip = hal_riscv::cpu::read_mip();
    let mie = hal_riscv::cpu::read_mie();
    let mcause = hal_riscv::cpu::read_mcause();
    let mstatus = hal_riscv::cpu::read_mstatus();
    let mtval = hal_riscv::cpu::read_mtval();
    let mepc = hal_riscv::cpu::read_mepc();

    crate::serial_debug!("{}", mstatus);
    crate::serial_debug!("{}", mie);
    crate::serial_debug!("{}", mip);
    crate::serial_debug!("mepc ::: {:?}", mepc);
    crate::serial_debug!("mtval ::: {:?}", mtval);
    crate::serial_debug!("M-mode: {}", mcause);
}

#[inline(always)]
pub fn dump_supervisor_registers() {
    let sstatus = hal_riscv::cpu::read_sstatus();
    let sie = hal_riscv::cpu::read_sie();
    let sip = hal_riscv::cpu::read_sip();
    let scause = hal_riscv::cpu::read_scause();

    crate::serial_debug!("{}", sstatus);
    crate::serial_debug!("{}", sie);
    crate::serial_debug!("{}", sip);
    crate::serial_debug!("S-mode: {}", scause);
}
