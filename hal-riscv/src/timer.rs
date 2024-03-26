use core::arch::asm;

const RISCV_MTIME_ADDR: u64 = 0x0200BFF8;
const RISCV_MTIMECMP_ADDR: u64 = 0x02004000;

#[inline(always)]
pub fn read_mtime() -> u64 {
    let mut mtime: u64;
    unsafe {
        asm!("ld {1}, 0({0})", in(reg) RISCV_MTIME_ADDR, out(reg) mtime);
    }

    mtime
}

#[inline(always)]
pub fn write_mtimecmp(mtime: u64) {
    unsafe {
        asm!("sd {0}, 0({1})", in(reg) mtime, in(reg) RISCV_MTIMECMP_ADDR);
    }
}
