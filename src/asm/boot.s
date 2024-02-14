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
    csrw              satp, zero                              # Disable paging

    la                a0, _bss_start                          # Initialize BSS section to zero
    la                a1, _bss_end
    bgeu              a0, a1, 2f

1:
    sd                zero, (a0)
    addi              a0, a0, 8
    bltu              a0, a1, 1b

2:
    la                sp, _stack_end                          # Prepare to switch to Rust-based entry code

    la                t0, machine_interrupt_handler_table
    addi              t0, t0, 1
    csrw              mtvec, t0
    li                t0, 1 << 5
    csrw              mideleg, t0


    la                ra, 3f                                  # Return location after Rust-based entry code returns
    call              kinit

3:
    li                t0, (0b01 << 11) | 1 << 5               # Set MPP to S-mode, enable SPIE
    csrw              mstatus, t0

    li                t0, 1 << 7 | 1 << 5                     # Enable machine & supervisor timer interrupt
    csrw              mie, t0

    csrwi             pmpcfg0, 0xf
    li                t0, 0x3fffffffffffff
    csrw              pmpaddr0, t0

    la                t1, main
    csrw              mepc, t1

# la t0, handle_supervisor_interrupt
# csrw stvec, t0

    mret

4:
    wfi
    j                 4b

    .balign           4
machine_interrupt_handler_table:
    .org              machine_interrupt_handler_table + 0*4
    jal               zero, exception_handler
    .org              machine_interrupt_handler_table + 1*4
    jal               zero, noop /* 1 */
    .org              machine_interrupt_handler_table + 3*4
    jal               zero, noop /* 3 */
    .org              machine_interrupt_handler_table + 5*4
    jal               zero, noop /* 5 */
    .org              machine_interrupt_handler_table + 7*4
    jal               zero, machine_timer_handler /* 7 */
    .org              machine_interrupt_handler_table + 9*4
    jal               zero, noop /* 9 */
    .org              machine_interrupt_handler_table + 11*4
    jal               zero, noop /* 11 */

noop:
    nop

exception_handler:
    write_serial_char 69
    write_serial_char 0xa

    j                 4b

machine_timer_handler:
    write_serial_char 73
    write_serial_char 0xa

# csrs mie, t0 # Enable STIE

    li                t0, 1 << 5                              # Enable STIP bit
    csrs              mip, t0

    li                t3, RISCV_MTIME_ADDR
    ld                t0, 0(t3)
    li                t2, RISCV_MTIMECMP_ADDR
    li                t1, 1000000
    add               t0, t0, t1
    sd                t0, 0(t2)

    mret
