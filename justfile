# bin := "target/riscv64gc-unknown-none-elf/debug/interrupts"
bin := "target/riscv64gc-unknown-none-elf/release/interrupts"

dump:
    @ riscv64-linux-objdump --disassemble --disassembler-color on -M no-aliases {{bin}}

dump-release:
    @ riscv64-linux-objdump --disassemble --disassembler-color on -M no-aliases \
        target/riscv64gc-unknown-none-elf/release/interrupts

run:
    @ qemu-system-riscv64 --machine virt --serial stdio --monitor none \
        --bios {{bin}} --nographic \
        -d guest_errors,unimp -D log.txt -m 128M

debug:
    @ qemu-system-riscv64 -s -S --machine virt --serial stdio --monitor none \
        --bios {{bin}} --nographic \
        -d guest_errors,unimp -D log.txt -m 128M

gdb:
    @ gdb-multiarch --init-command cmds.gdb

build:
    cargo build --release 

test:
    cargo test --target riscv64gc-unknown-none-elf --release 

clippy:
    cargo clippy --fix


