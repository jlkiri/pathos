use core::arch::asm;

#[repr(u8)]
#[derive(Debug)]
pub enum Ecall {
    SModeFinishBootstrap,
    ClearPendingInterrupt(u8),
    Exit(u8),
}

pub fn ecall(call: Ecall) {
    match call {
        Ecall::SModeFinishBootstrap => unsafe { asm!("ecall", in("x30") 1) },
        Ecall::ClearPendingInterrupt(cause) => unsafe {
            asm!("ecall", in("x30") 2, in("x31") cause)
        },
        Ecall::Exit(code) => unsafe { asm!("ecall", in("x30") 3, in("x31") code) },
    }
}

pub fn read_ecall() -> Ecall {
    let ecall: u8;
    let payload: u8;
    unsafe {
        asm!("", out("x30") ecall, out("x31") payload);
    }

    match ecall {
        1 => Ecall::SModeFinishBootstrap,
        2 => Ecall::ClearPendingInterrupt(payload),
        3 => Ecall::Exit(payload),
        _ => panic!("Unknown ecall: {}", ecall),
    }
}
