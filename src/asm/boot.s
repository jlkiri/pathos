    .equ              RISCV_MTIMECMP_ADDR, 0x2000000 + 0x4000
    .equ              RISCV_MTIME_ADDR, 0x2000000 + 0xBFF8

    .option           norvc

    .macro            write_serial_char char
    li                t0, \char
    li                t1, 0x10000000
    sb                t0, (t1)
    .endm

    .section          .text.boot
    .global           _start
_start:
    csrw              satp, zero                                   # Disable paging

    la                a0, _bss_start                               # Initialize BSS section to zero
    la                a1, _bss_end
    bgeu              a0, a1, 2f

1:
    sd                zero, (a0)
    addi              a0, a0, 8
    bltu              a0, a1, 1b

2:
    la                sp, _stack_end                               # Prepare to switch to Rust-based entry code

    csrwi             pmpcfg0, 0xf                                 # Let S-mode access all physical memory
    li                t0, 0x3fffffffffffff
    csrw              pmpaddr0, t0

    call kinit

    la                t0, machine_interrupt_handler
    csrw              mtvec, t0

    la                t0, main
    csrw              mepc, t0
    mret

    .balign           4
machine_interrupt_handler:
    csrr              t0, mcause

    li                t2, 0x8000000000000007                       # == Machine timer interrupt
    beq               t0, t2, machine_timer_handler

    li                t2, 0x9                                      # == S-mode ECALL
    beq               t0, t2, ecall_handler

    call m_panic

    mret

ecall_handler:
    li                t0, 1
    beq               t0, x31, setup_ecall_handler

    li                t0, 2
    beq               t0, x31, clear_stip_ecall_handler

    call m_panic


setup_ecall_handler:
    li                t0, (1 << 5) | (1 << 7)
    csrw              mie, t0

    li                t0, (0b01 << 11) | (1 << 7) | (1 << 13)      # Set MPP to S-mode, enable MPIE, and FS (which is needed to enable floating point load/store instructions)
    csrs              mstatus, t0

    csrr              t0, mepc
    addi              t0, t0, 4                                    # Return to next instruction after ECALL
    csrw              mepc, t0

    mv                x31, zero

# write_serial_char 0x24 # Print '$'
    mret

clear_stip_ecall_handler:
    li                t0, 1 << 5
    csrc              mip, t0

    csrr              t0, mepc
    addi              t0, t0, 4                                    # Return to next instruction after ECALL
    csrw              mepc, t0

    mv                x31, zero

    mret

machine_timer_handler:
    li                t3, RISCV_MTIME_ADDR
    ld                t0, 0(t3)
    li                t2, RISCV_MTIMECMP_ADDR
    li                t1, 10000000
    add               t0, t0, t1
    sd                t0, 0(t2)

    li                t0, 1 << 5                                   # Enable STIP bit to let S-mode handle the interrupt
    csrs              mip, t0

    mret

loop:
    j                 loop