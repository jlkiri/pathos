use core::arch::asm;
use core::marker::FnPtr;
use core::panic::PanicInfo;

use crate::serial_info;

type HandlerFunc = extern "riscv-interrupt-s" fn();

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
        hal::cpu::write_stvec_vectored(Self::stvec_table);
        unsafe { asm!("li x31, 1", "ecall") }
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
    fn stvec_table() {
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

extern "riscv-interrupt-s" fn noop() {}

pub extern "riscv-interrupt-s" fn dispatch_smode_interrupt() {
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
