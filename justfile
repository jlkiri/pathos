bin := "target/riscv64gc-unknown-none-elf/debug/interrupts"

dump:
    @ riscv64-linux-objdump --disassemble --disassembler-color on {{bin}}

run:
    @ qemu-system-riscv64 --machine virt --serial stdio --monitor none \
        --bios fw_dynamic.elf --kernel {{bin}} --nographic