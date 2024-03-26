use core::{fmt, ptr};

type MmioAddr = usize;
type PID = u8;

const UART_MMIO_ADDR: MmioAddr = 0x10_0000_0000;

pub struct Serial(MmioAddr, PID);

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            unsafe { ptr::write_volatile(self.0 as *mut u8, byte) }
        }
        Ok(())
    }
}

impl Serial {
    pub fn new(pid: u8) -> Self {
        Serial(UART_MMIO_ADDR, pid)
    }

    pub fn print_pid(&self) {
        unsafe { ptr::write_volatile(self.0 as *mut u8, self.1 + 48) }
        unsafe { ptr::write_volatile(self.0 as *mut u8, ' ' as u8) }
        unsafe { ptr::write_volatile(self.0 as *mut u8, ':' as u8) }
        unsafe { ptr::write_volatile(self.0 as *mut u8, ':' as u8) }
        unsafe { ptr::write_volatile(self.0 as *mut u8, ':' as u8) }
        unsafe { ptr::write_volatile(self.0 as *mut u8, ' ' as u8) }
    }
}
