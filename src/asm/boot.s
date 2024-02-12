    .option  norvc

    .section .text.boot
    .global  _start

_start:
    csrw     satp, zero

    la       a0, _bss_start
    la       a1, _bss_end
    bgeu     a0, a1, 2f

1:
    sd       zero, (a0)
    addi     a0, a0, 8
    bltu     a0, a1, 1b

2:
    la       sp, _stack_end

    la       t0, machine_interrupt_handler_table
    addi     t0, t0, 1
    csrw     mtvec, t0
    li       t0, 1 << 5
    csrw     mideleg, t0
    li       t0, 1 << 7
    csrw     mie, t0

    la       t1, main
    csrw     mepc, t1

    li       t0, 1 << 11                           # Set MPP to S-mode
    csrw     mstatus, t0

    jal      write_uart_byte
    csrwi    pmpcfg0, 0xf
    li       t0, -1
    csrw     pmpaddr0, t0

    mret

    la       ra, 3f

3:
    li       t0, 0x69
    li       t1, 0x10000000
    sb       t0, (t1)

write_uart_byte:
    li       t0, 0x68
    li       t1, 0x10000000
    sb       t0, (t1)
    ret

    .balign  4
machine_interrupt_handler_table:
    .org     machine_interrupt_handler_table + 7*4
    jal      zero, machine_timer_handler /* 7 */

machine_timer_handler:
# jal write_uart_byte
    li       t0, 1 << 5
    csrw     mip, t0                               # Enable STIP bit
    csrs     mie, t0                               # Enable STIE
    li       t0, 1 << 11 | (1 << 13) | 1 << 3
    csrw     mstatus, t0
    mret

# la t2, mtvec_table
# addi t2, t2, 1 # Enable vector mode
# csrw stvec, t2 # Init S-mode interrupt vector table

# li t0, 32
# csrs sie, t0 # Enable supervisor timer interrupt (5)
# csrsi sstatus, 2 # Enable supervisor interrupt handling

# jal s_set_timer

# la t1, u_cause_unhandled_ecall
# # la t1, user_loop
# csrw sepc, t1 # Set U-mode routine address to jump to after switching to U-mode

# la t0, _L3_PAGETBL_PTR # Set t0 to L3 page table address
# li t1, (1 << 4) | 0b111 # Create fake L3 PTE (set U-bit and R & W bits)
# sd t1, 0(t0)

# srli t0, t0, 12 # Convert page table address to PPN (div by 4096)
# # li t2, 0x8000000000000000 # Create a valid satp register value (set MODE to 8 and OR with page table PPN)
# li t2, 8 << 60 # Create a valid satp register value (set MODE to 8 and OR with page table PPN)
# or t2, t2, t0

# jal print_here

# csrw satp, t2 # Enable sv39 paging

# jal print_here

# sret # Switch to U-mode

# j loop

# .balign 4
# stvec_table:
# .org stvec_table + 0*4
# jal zero, exception_handler /* 0 */
# .org stvec_table + 1*4
# jal zero, noop /* 1 */
# .org stvec_table + 3*4
# jal zero, noop /* 3 */
# .org stvec_table + 5*4
# jal zero, s_handle_timer_interrupt /* 5 */
# .org stvec_table + 7*4
# jal zero, noop /* 7 */
# .org stvec_table + 9*4
# jal zero, noop /* 9 */
# .org stvec_table + 11*4
# jal zero, noop /* 11 */

# loop:
# wfi
# j loop
