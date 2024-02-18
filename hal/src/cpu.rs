use core::{arch::asm, fmt};

pub enum Cause {
    Interrupt(u8),
    Exception(u8),
}

#[derive(Default)]
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

pub struct Mie {
    pub ssie: u8,
    pub stie: u8,
    pub mtie: u8,
    pub msie: u8,
}

pub struct Mip {
    pub ssip: u8,
    pub stip: u8,
    pub mtip: u8,
    pub msip: u8,
}

#[derive(Default)]
pub struct Mideleg {
    pub ssi: u8,
    pub sti: u8,
    pub mti: u8,
    pub msi: u8,
}

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
            "csrw mip, {}",
            in(reg) mip
        )
    }
}

#[inline(always)]
pub fn write_mepc(addr: *mut u8) {
    unsafe {
        asm!(
            "csrw mepc, {}",
            in(reg) addr
        )
    }
}

#[inline(always)]
pub fn write_mepc_next(addr: u64) {
    unsafe {
        asm!(
            "addi {0}, {0}, 4",
            "csrw mepc, {0}",
            in(reg) addr
        )
    }
}

#[inline(always)]
pub fn write_mtvec(fun: fn()) {
    unsafe {
        asm!(
            "csrw mtvec, {}",
            in(reg) fun
        )
    }
}

#[inline(always)]
pub fn write_stvec_vectored(fun: fn()) {
    unsafe {
        asm!(
            "addi {0}, {0}, 1",
            "csrw stvec, {0}",
            in(reg) fun
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
        1 => Cause::Exception(cause as u8),
        -1 => Cause::Interrupt(cause as u8),
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
        1 => Cause::Exception(cause as u8),
        -1 => Cause::Interrupt(cause as u8),
        _ => unreachable!(),
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
            Cause::Interrupt(cause) => write!(f, "INT cause # {}", cause),
            Cause::Exception(cause) => write!(f, "EXC cause # {}", cause),
        }
    }
}
