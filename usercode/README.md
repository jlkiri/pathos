riscv64-linux-as  -march=rv64gc -mabi=lp64d -o rt.o -c rt.s
riscv64-linux-ld app.o rt.o --no-dynamic-linker -m elf64lriscv -nostdlib -s -o app
riscv64-linux-objdump -D target/riscv64gc-unknown-none-elf/release/usercode -j .text | grep -C 10 _start

rustflags = "-C link-arg=rt.o"

```rs
    // let mut small_rng: SmallRng = SmallRng::seed_from_u64(12345);
    // let mut serial: Serial = Serial(UART_MMIO_ADDR);

    loop {
        // let num = small_rng.next_u32();
        // serial.write_fmt(format_args!("{}", num)).expect("nzzzzz");
        // if num < 10 {
        // serial.write_str(".").expect("noooooooo");
        // }
        // unsafe { asm!("ebreak") }
    }
```