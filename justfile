bin := "target/riscv64gc-unknown-none-elf/release/pathos"

dump:
    @ cargo objdump -q --release --bin pathos -- --disassemble \
        --no-show-raw-insn -M no-aliases

run:
    @ qemu-system-riscv64 --machine virt --smp 1 --cpu rv64 --serial stdio --monitor none \
        --bios {{bin}} --nographic \
        -d guest_errors,unimp -D log.txt -m 128M

debug:
    @ qemu-system-riscv64 -s -S --machine virt --serial stdio --monitor none \
        --bios {{bin}} --nographic \
        -d guest_errors,unimp -D log.txt -m 128M

gdb:
    @ gdb-multiarch --init-command cmds.gdb

build:
    @ cargo build --release

clean:
    @ cargo clean

test:
    @ cargo test --target x86_64-unknown-linux-gnu



