riscv64-linux-as  -march=rv64gc -mabi=lp64d -o rt.o -c rt.s
riscv64-linux-ld app.o rt.o --no-dynamic-linker -m elf64lriscv -nostdlib -s -o app
riscv64-linux-objdump -D target/riscv64gc-unknown-none-elf/release/usercode -j .text | grep -C 10 _start

rustflags = "-C link-arg=rt.o"