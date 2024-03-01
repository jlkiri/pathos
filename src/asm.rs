use core::arch::global_asm;

global_asm!(include_str!("asm/boot.s"));
global_asm!(include_str!("asm/mem.s"));
