#![no_std]
#![no_main]

mod print;

use core::{fmt::Write, panic::PanicInfo};

use print::Serial;

#[no_mangle]
fn main(pid: u8) -> core::result::Result<(), core::fmt::Error> {
    let mut serial = Serial::new(pid);
    serial.print_pid();
    writeln!(serial, "Greetings from userspace!")?;
    writeln!(
        serial,
        "This program prints its TID (task ID) in an endless loop!"
    )?;

    let mut total = 0;
    loop {
        total = add(total, 1);
        write_pid_if_divisible_by(pid, total, 854884, &mut serial)?;
    }

    Ok(())
}

#[inline(never)]
fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[inline(never)]
fn write_pid_if_divisible_by(
    pid: u8,
    num: u32,
    divisor: u32,
    serial: &mut Serial,
) -> core::result::Result<(), core::fmt::Error> {
    if num % divisor == 0 {
        serial.write_char(char::from_digit(pid as u32, 10).unwrap())?;
    }
    Ok(())
}

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
