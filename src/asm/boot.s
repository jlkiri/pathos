    .option  norvc

    .section .text.boot
    .global  _start
_start:
    csrw     satp, zero                # Disable paging

    la       a0, _bss_start            # Initialize BSS section to zero
    la       a1, _bss_end
    bgeu     a0, a1, 2f

1:
    sd       zero, (a0)
    addi     a0, a0, 8
    bltu     a0, a1, 1b

2:
    la       sp, _stack_end            # Prepare to switch to Rust-based entry code

    csrwi    pmpcfg0, 0xf              # Let S-mode access all physical memory
    li       t0, 0xffffffffffffff >> 2
    csrw     pmpaddr0, t0

    call     kinit
