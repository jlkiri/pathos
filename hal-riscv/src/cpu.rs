use core::{arch::asm, fmt};

#[derive(Debug, Clone)]
pub enum Cause {
    Interrupt(Interrupt),
    Exception(Exception),
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Interrupt {
    SupervisorSoftware = 1,
    MachineSoftware = 3,
    SupervisorTimer = 5,
    MachineTimer = 7,
    SupervisorExternal = 9,
    MachineExternal = 11,
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Exception {
    InstructionMisaligned,
    InstructionFault,
    IllegalInstruction,
    Breakpoint,
    LoadMisaligned,
    LoadFault,
    StoreMisaligned,
    StoreFault,
    UserEcall,
    SupervisorEcall,
    MachineEcall,
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
}

impl From<u8> for Interrupt {
    fn from(value: u8) -> Self {
        match value {
            1 => Interrupt::SupervisorSoftware,
            3 => Interrupt::MachineSoftware,
            5 => Interrupt::SupervisorTimer,
            7 => Interrupt::MachineTimer,
            9 => Interrupt::SupervisorExternal,
            11 => Interrupt::MachineExternal,
            _ => unreachable!(),
        }
    }
}

impl From<u8> for Exception {
    fn from(value: u8) -> Self {
        match value {
            0 => Exception::InstructionMisaligned,
            1 => Exception::InstructionFault,
            2 => Exception::IllegalInstruction,
            3 => Exception::Breakpoint,
            4 => Exception::LoadMisaligned,
            5 => Exception::LoadFault,
            6 => Exception::StoreMisaligned,
            7 => Exception::StoreFault,
            8 => Exception::UserEcall,
            9 => Exception::SupervisorEcall,
            11 => Exception::MachineEcall,
            12 => Exception::InstructionPageFault,
            13 => Exception::LoadPageFault,
            15 => Exception::StorePageFault,
            _ => unreachable!(),
        }
    }
}

#[derive(Default, Clone)]
pub struct Mstatus {
    pub sie: u8,
    pub mie: u8,
    pub upie: u8,
    pub spie: u8,
    pub mpie: u8,
    pub spp: u8,
    pub mpp: u8,
    pub fs: u8,
}

#[derive(Default, Clone)]
pub struct Mie {
    pub ssie: u8,
    pub stie: u8,
    pub mtie: u8,
    pub msie: u8,
}

#[derive(Default, Clone)]
pub struct Mip {
    pub ssip: u8,
    pub stip: u8,
    pub mtip: u8,
    pub msip: u8,
}

#[derive(Default, Clone)]
pub struct Mideleg {
    pub ssi: u8,
    pub sti: u8,
    pub mti: u8,
    pub msi: u8,
}

#[derive(Default, Clone)]
pub struct Medeleg {
    pub uecall: u8,
}

#[derive(Debug, Default)]
pub struct Sstatus {
    pub sie: u8,
    pub spie: u8,
    pub spp: u8,
}

pub struct Sip {
    pub ssip: u8,
    pub stip: u8,
}

pub struct Sie {
    pub ssie: u8,
    pub stie: u8,
}

#[derive(Debug, Clone)]
pub struct Satp {
    mode: u64,
    ppn: u64,
}

impl Satp {
    pub fn new(mode: u64, addr: usize) -> Self {
        let ppn = (addr >> 12) as u64;
        Self { mode, ppn }
    }
}

#[inline(always)]
pub fn read_sip() -> Sip {
    let sip: u64;
    unsafe {
        asm!(
            "csrr {}, sip",
            out(reg) sip
        )
    }

    Sip {
        ssip: (sip >> 1) as u8 & 1,
        stip: (sip >> 5) as u8 & 1,
    }
}

#[inline(always)]
pub fn read_sie() -> Sie {
    let sie: u64;
    unsafe {
        asm!(
            "csrr {}, sie",
            out(reg) sie
        )
    }

    Sie {
        ssie: (sie >> 1) as u8 & 1,
        stie: (sie >> 5) as u8 & 1,
    }
}

#[inline(always)]
pub fn write_satp(satp: Satp) {
    let satp = (satp.mode << 60) | satp.ppn;
    unsafe {
        asm!(
            "csrw satp, {}",
            "sfence.vma x0, x0",
            in(reg) satp
        )
    }
}

#[inline(always)]
pub fn write_sscratch(addr: usize) {
    unsafe {
        asm!(
            "csrw sscratch, {}",
            in(reg) addr
        )
    }
}

#[inline(always)]
pub fn read_sscratch() -> *const u8 {
    let sscratch: *const u8;
    unsafe {
        asm!(
            "csrr {}, sscratch",
            out(reg) sscratch
        )
    }

    sscratch
}

#[inline(always)]
pub fn write_sp(addr: *const u8) {
    unsafe {
        asm!(
            "mv sp, {}",
            in(reg) addr
        )
    }
}

#[inline(always)]
pub fn read_sp() -> usize {
    let sp: usize;
    unsafe {
        asm!(
            "mv {}, sp",
            out(reg) sp
        )
    }

    sp
}

#[inline(always)]
pub fn read_mip() -> Mip {
    let mip: u64;
    unsafe {
        asm!(
            "csrr {}, mip",
            out(reg) mip
        )
    }

    Mip {
        ssip: (mip >> 1) as u8 & 1,
        stip: (mip >> 5) as u8 & 1,
        mtip: (mip >> 7) as u8 & 1,
        msip: (mip >> 3) as u8 & 1,
    }
}

#[inline(always)]
pub fn read_mie() -> Mie {
    let mie: u64;
    unsafe {
        asm!(
            "csrr {}, mie",
            out(reg) mie
        )
    }

    Mie {
        ssie: (mie >> 1) as u8 & 1,
        stie: (mie >> 5) as u8 & 1,
        mtie: (mie >> 7) as u8 & 1,
        msie: (mie >> 3) as u8 & 1,
    }
}

#[inline(always)]
pub fn read_mstatus() -> Mstatus {
    let mstatus: u64;
    unsafe {
        asm!(
            "csrr {}, mstatus",
            out(reg) mstatus
        )
    }

    Mstatus {
        sie: (mstatus >> 1) as u8 & 1,
        mie: (mstatus >> 3) as u8 & 1,
        upie: (mstatus >> 4) as u8 & 1,
        spie: (mstatus >> 5) as u8 & 1,
        mpie: (mstatus >> 7) as u8 & 1,
        spp: (mstatus >> 8) as u8 & 1,
        mpp: (mstatus >> 11) as u8 & 3,
        fs: (mstatus >> 13) as u8 & 3,
    }
}

#[inline(always)]
pub fn read_mtval() -> *const () {
    let mtval: *const ();
    unsafe {
        asm!(
            "csrr {}, mtval",
            out(reg) mtval
        )
    }

    mtval
}

#[inline(always)]
pub fn read_stval() -> *const () {
    let stval: *const ();
    unsafe {
        asm!(
            "csrr {}, stval",
            out(reg) stval
        )
    }

    stval
}

#[inline(always)]
pub fn write_mstatus(mstatus: Mstatus) {
    let mstatus = (mstatus.sie as u64) << 1
        | (mstatus.mie as u64) << 3
        | (mstatus.upie as u64) << 4
        | (mstatus.spie as u64) << 5
        | (mstatus.mpie as u64) << 7
        | (mstatus.spp as u64) << 8
        | (mstatus.mpp as u64) << 11
        | (mstatus.fs as u64) << 13;
    unsafe {
        asm!(
            "csrw mstatus, {}",
            in(reg) mstatus
        )
    }
}

#[inline(always)]
pub fn read_mideleg() -> Mideleg {
    let mideleg: u64;
    unsafe {
        asm!(
            "csrr {}, mideleg",
            out(reg) mideleg
        )
    }

    Mideleg {
        ssi: (mideleg >> 1) as u8 & 1,
        sti: (mideleg >> 5) as u8 & 1,
        mti: (mideleg >> 7) as u8 & 1,
        msi: (mideleg >> 3) as u8 & 1,
    }
}

#[inline(always)]
pub fn write_mideleg(mideleg: Mideleg) {
    let mideleg = (mideleg.ssi as u64) << 1
        | (mideleg.sti as u64) << 5
        | (mideleg.mti as u64) << 7
        | (mideleg.msi as u64) << 3;
    unsafe {
        asm!(
            "csrw mideleg, {}",
            in(reg) mideleg
        )
    }
}

#[inline(always)]
pub fn write_medeleg(medeleg: Medeleg) {
    let medeleg = (medeleg.uecall as u64) << 8;
    unsafe {
        asm!(
            "csrw medeleg, {}",
            in(reg) medeleg
        )
    }
}

#[inline(always)]
pub fn write_mie(mie: Mie) {
    let mie = (mie.ssie as u64) << 1
        | (mie.stie as u64) << 5
        | (mie.mtie as u64) << 7
        | (mie.msie as u64) << 3;
    unsafe {
        asm!(
            "csrw mie, {}",
            in(reg) mie
        )
    }
}

#[inline(always)]
pub fn write_mip(mip: Mip) {
    let mip = (mip.ssip as u64) << 1
        | (mip.stip as u64) << 5
        | (mip.mtip as u64) << 7
        | (mip.msip as u64) << 3;

    unsafe {
        asm!(
            "csrs mip, {}",
            in(reg) mip
        )
    }
}

#[inline(always)]
pub fn clear_mip(bit: u8) {
    unsafe { asm!("csrc mip, {}", in(reg) 1 << bit) }
}

#[inline(always)]
pub fn read_mepc() -> *const () {
    let mepc: *const ();
    unsafe {
        asm!(
            "csrr {}, mepc",
            out(reg) mepc
        )
    }

    mepc
}

#[inline(always)]
pub fn read_sepc() -> *const () {
    let sepc: *const ();
    unsafe {
        asm!(
            "csrr {}, sepc",
            out(reg) sepc
        )
    }

    sepc
}

#[inline(always)]
pub fn write_mepc(addr: *const ()) {
    unsafe {
        asm!(
            "csrw mepc, {}",
            in(reg) addr
        )
    }
}

#[inline(always)]
pub fn write_sepc(addr: *const ()) {
    unsafe {
        asm!(
            "csrw sepc, {}",
            in(reg) addr
        )
    }
}

#[inline(always)]
pub fn write_mepc_next() {
    unsafe { asm!("csrr t0, mepc", "addi t0, t0, 4", "csrw mepc, t0") }
}

#[inline(always)]
pub fn write_mtvec_vectored(fun: fn()) {
    unsafe {
        asm!(
            "addi {0}, {0}, 1",
            "csrw mtvec, {0}",
            in(reg) fun
        )
    }
}

#[inline(always)]
pub fn write_stvec_vectored(addr: fn()) {
    unsafe {
        asm!(
            "addi {0}, {0}, 1",
            "csrw stvec, {0}",
            in(reg) addr
        )
    }
}

#[inline(always)]
pub fn read_sstatus() -> Sstatus {
    let sstatus: u64;
    unsafe {
        asm!(
            "csrr {}, sstatus",
            out(reg) sstatus
        )
    }

    Sstatus {
        sie: (sstatus >> 1) as u8 & 1,
        spie: (sstatus >> 5) as u8 & 1,
        spp: (sstatus >> 8) as u8 & 1,
    }
}

#[inline(always)]
pub fn set_sstatus(sstatus: Sstatus) {
    let sstatus =
        (sstatus.sie as u64) << 1 | (sstatus.spie as u64) << 5 | (sstatus.spp as u64) << 8;
    unsafe {
        asm!(
            "csrs sstatus, {}",
            in(reg) sstatus
        )
    }
}

#[inline(always)]
pub fn read_scause() -> Cause {
    let scause: u64;
    unsafe {
        asm!(
            "csrr {}, scause",
            out(reg) scause
        )
    }

    let cause = scause as i64;
    match cause.signum() {
        0 | 1 => Cause::Exception(Exception::from(cause as u8)),
        -1 => Cause::Interrupt(Interrupt::from(cause as u8)),
        _ => unreachable!(),
    }
}

#[inline(always)]
pub fn read_mcause() -> Cause {
    let mcause: u64;
    unsafe {
        asm!(
            "csrr {}, mcause",
            out(reg) mcause
        )
    }

    let cause = mcause as i64;
    match cause.signum() {
        1 => Cause::Exception(Exception::from(cause as u8)),
        -1 => Cause::Interrupt(Interrupt::from(cause as u8)),
        _ => unreachable!(),
    }
}

impl fmt::Display for Satp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "satp ::: mode: {}, ppn: 0x{:x} (0x{:x})",
            self.mode,
            self.ppn,
            self.mode << 60 | self.ppn
        )
    }
}

impl fmt::Display for Mstatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mstatus ::: sie: {:b}, mie: {:b}, upie: {:b}, spie: {:b}, mpie: {:b}, spp: {:b}, mpp: {:b}, fs: {:b}",
            self.sie, self.mie, self.upie, self.spie, self.mpie, self.spp, self.mpp, self.fs
        )
    }
}

impl fmt::Display for Mie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mie ::: ssie: {:b}, stie: {:b}, mtie: {:b}, msie: {:b}",
            self.ssie, self.stie, self.mtie, self.msie
        )
    }
}

impl fmt::Display for Mip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mip ::: ssip: {:b}, stip: {:b}, mtip: {:b}, msip: {:b}",
            self.ssip, self.stip, self.mtip, self.msip
        )
    }
}

impl fmt::Display for Mideleg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mideleg ::: ssi: {:b}, sti: {:b}, mti: {:b}, msi: {:b}",
            self.ssi, self.sti, self.mti, self.msi
        )
    }
}

impl fmt::Display for Medeleg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "medeleg ::: uecall: {:b}", self.uecall)
    }
}

impl fmt::Display for Sstatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "sstatus ::: sie: {:b}, spie: {:b}, spp: {:b}",
            self.sie, self.spie, self.spp
        )
    }
}

impl fmt::Display for Sip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sip ::: ssip: {:b}, stip: {:b}", self.ssip, self.stip)
    }
}

impl fmt::Display for Sie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sie ::: ssie: {:b}, stie: {:b}", self.ssie, self.stie)
    }
}

impl fmt::Display for Cause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cause::Interrupt(cause) => {
                write!(f, "[INT] cause {:?} = {}", cause, (*cause).clone() as u8)
            }
            Cause::Exception(cause) => {
                write!(f, "[EXC] cause {:?} = {}", cause, (*cause).clone() as u8)
            }
        }
    }
}
