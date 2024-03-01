# bin := "target/riscv64gc-unknown-none-elf/debug/pathos"
bin := "target/riscv64gc-unknown-none-elf/release/pathos"
cargo_unstable_flags := "-Z build-std=core,alloc"

dump:
    @ riscv64-linux-objdump --no-show-raw-insn --disassemble --disassembler-color on -M no-aliases {{bin}}

dump2:
    cargo objdump --release --bin pathos -- --disassemble \
        --no-show-raw-insn --demangle -s --section .text.vector

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
    cargo build {{cargo_unstable_flags}} --release

clean:
    cargo clean

test:
    cargo test --target x86_64-unknown-linux-gnu



