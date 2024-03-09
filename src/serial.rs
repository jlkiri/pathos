use core::{fmt, ptr};
use owo_colors::OwoColorize;
use spin::Mutex;

const UART_MMIO_ADDR: usize = 0x10000000;

const INFO: &str = "INFO";
const DEBUG: &str = "DEBUG";
const ERROR: &str = "ERROR";

const PATHOS: &str = "PathOS";

struct Serial(usize);

static SERIAL: Mutex<Serial> = Mutex::new(Serial(UART_MMIO_ADDR));

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            unsafe { ptr::write_volatile(self.0 as *mut u8, byte) }
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _info(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    let mut serial = SERIAL.lock();

    serial
        .write_fmt(format_args!("{} ", PATHOS.blue()))
        .expect("Printing to serial failed");

    serial
        .write_fmt(format_args!("[{}] ", INFO.bright_green()))
        .expect("Printing to serial failed");

    serial.write_fmt(args).expect("Printing to serial failed");
}

#[doc(hidden)]
pub fn _error(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    let mut serial = SERIAL.lock();

    serial
        .write_fmt(format_args!("{} ", PATHOS.blue()))
        .expect("Printing to serial failed");

    serial
        .write_fmt(format_args!("[{}] ", ERROR.bright_red()))
        .expect("Printing to serial failed");

    serial.write_fmt(args).expect("Printing to serial failed");
}

#[doc(hidden)]
pub fn _debug(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    let mut serial = SERIAL.lock();

    serial
        .write_fmt(format_args!("{} ", PATHOS.blue()))
        .expect("Printing to serial failed");

    serial
        .write_fmt(format_args!("[{}] ", DEBUG.bright_cyan()))
        .expect("Printing to serial failed");

    serial.write_fmt(args).expect("Printing to serial failed");
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

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
