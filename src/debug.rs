use crate::SCHEDULER;

#[inline(always)]
pub fn dump_machine_registers() {
    let mip = hal_riscv::cpu::read_mip();
    let mie = hal_riscv::cpu::read_mie();
    let mstatus = hal_riscv::cpu::read_mstatus();
    let mtval = hal_riscv::cpu::read_mtval();
    let mepc = hal_riscv::cpu::read_mepc();

    crate::serial_debug!("{}", mstatus);
    crate::serial_debug!("{}", mie);
    crate::serial_debug!("{}", mip);
    crate::serial_debug!("mepc ::: {:?}", mepc);
    crate::serial_debug!("mtval ::: {:?}", mtval);
    crate::serial_debug!("sp ::: {:?}", hal_riscv::cpu::read_sp());
}

#[inline(always)]
pub unsafe fn dump_trap_frame() {
    let guard = SCHEDULER.lock();
    let scheduler = guard.get().expect("Scheduler not initialized");
    let current = scheduler.task(scheduler.current());
    crate::serial_debug!("{:x?}", current.trap_frame);
}

#[inline(always)]
pub fn dump_supervisor_registers() {
    let sstatus = hal_riscv::cpu::read_sstatus();
    let sie = hal_riscv::cpu::read_sie();
    let sip = hal_riscv::cpu::read_sip();
    let stval = hal_riscv::cpu::read_stval();
    let sepc = hal_riscv::cpu::read_sepc();

    crate::serial_debug!("{}", sstatus);
    crate::serial_debug!("{}", sie);
    crate::serial_debug!("{}", sip);
    crate::serial_debug!("sepc ::: {:?}", sepc);
    crate::serial_debug!("stval ::: {:?}", stval);
}
