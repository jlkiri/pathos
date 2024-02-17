use core::arch::asm;
use core::marker::FnPtr;
use core::panic::PanicInfo;

use crate::serial_println;

type HandlerFunc = extern "riscv-interrupt-s" fn();

enum Cause {
    Interrupt(u8),
    Exception(u8),
}

static mut Handlers: [HandlerFunc; 12] = [noop; 12];

pub struct InterruptVectorTable {}

#[repr(u8)]
pub enum InterruptIndex {
    SupervisorSoftware = 1,
    MachineSoftware = 3,
    SupervisorTimer = 5,
    MachineTimer = 7,
    SupervisorExternal = 9,
    MachineExternal = 11,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        self.as_u8() as usize
    }
}

impl InterruptVectorTable {
    #[inline(always)]
    pub fn init(&self) {
        unsafe {
            asm!(
                "addi t0, t0, 1", // Enable vectored mode
                "csrw stvec, t0",
                "li x31, 1",
                "ecall",
                in("t0") Self::stvec_table
            )
        }
    }

    #[no_mangle]
    #[repr(align(4))]
    #[inline(always)]
    pub fn handle_sti(&self) {
        unsafe { Handlers[InterruptIndex::SupervisorTimer.as_usize()]() }
    }

    pub fn register_handler(&mut self, index: InterruptIndex, handler: HandlerFunc) {
        unsafe { Handlers[index.as_usize()] = handler }
    }

    #[no_mangle]
    #[naked]
    #[repr(align(4))]
    fn stvec_table(&self) {
        unsafe {
            asm!(
                ".org {1} + 1 * 4",
                "jal {0}",
                ".org {1} + 3 * 4",
                "jal {0}",
                ".org {1} + 5 * 4",
                "jal {2}",
                ".org {1} + 7 * 4",
                "jal {0}",
                ".org {1} + 9 * 4",
                "jal {0}",
                ".org {1} + 11 * 4",
                "jal {0}",
                sym noop,
                sym InterruptVectorTable::stvec_table,
                sym InterruptVectorTable::handle_sti,
                options(noreturn)
            )
        }
    }
}

#[inline(always)]
fn read_scause() -> Cause {
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

extern "riscv-interrupt-s" fn noop() {}

pub extern "riscv-interrupt-s" fn dispatch_smode_interrupt() {
    match read_scause() {
        Cause::Interrupt(5) => {
            crate::ok("Handle pending software timer interrupt");
            unsafe { asm!("li x31, 2", "ecall") }
        }
        _ => panic!(),
    }
}
