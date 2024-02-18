use core::{fmt, ptr};
use owo_colors::OwoColorize;

const UART_ADDR: usize = 0x10000000;

const INFO: &'static str = "INFO";
const DEBUG: &'static str = "DEBUG";
const ERROR: &'static str = "ERROR";

const PATHOS: &'static str = "PathOS";

struct Serial(*mut u8);

static mut SERIAL: Serial = Serial(UART_ADDR as *mut u8);

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            unsafe { ptr::write_volatile(self.0, byte) }
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _info(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    unsafe {
        SERIAL
            .write_fmt(format_args!("{} ", PATHOS.blue()))
            .expect("Printing to serial failed");

        SERIAL
            .write_fmt(format_args!("[{}] ", INFO.bright_green()))
            .expect("Printing to serial failed");

        SERIAL.write_fmt(args).expect("Printing to serial failed");
    }
}

#[doc(hidden)]
pub fn _error(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    unsafe {
        SERIAL
            .write_fmt(format_args!("{} ", PATHOS.blue()))
            .expect("Printing to serial failed");

        SERIAL
            .write_fmt(format_args!("[{}] ", ERROR.bright_red()))
            .expect("Printing to serial failed");

        SERIAL.write_fmt(args).expect("Printing to serial failed");
    }
}

#[doc(hidden)]
pub fn _debug(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    unsafe {
        SERIAL
            .write_fmt(format_args!("{} ", PATHOS.blue()))
            .expect("Printing to serial failed");

        SERIAL
            .write_fmt(format_args!("[{}] ", DEBUG.bright_cyan()))
            .expect("Printing to serial failed");

        SERIAL.write_fmt(args).expect("Printing to serial failed");
    }
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! _serial_info {
    ($($arg:tt)*) => {
        $crate::serial::_info(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_info {
    () => ($crate::_serial_info!("\n"));
    ($fmt:expr) => ($crate::_serial_info!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::_serial_info!(
        concat!($fmt, "\n"), $($arg)*));
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! _serial_error {
    ($($arg:tt)*) => {
        $crate::serial::_error(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_error {
    () => ($crate::_serial_error!("\n"));
    ($fmt:expr) => ($crate::_serial_error!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::_serial_error!(
        concat!($fmt, "\n"), $($arg)*));
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! _serial_debug {
    ($($arg:tt)*) => {
        $crate::serial::_debug(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_debug {
    () => ($crate::_serial_debug!("\n"));
    ($fmt:expr) => ($crate::_serial_debug!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::_serial_debug!(
        concat!($fmt, "\n"), $($arg)*));
}
