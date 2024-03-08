    .globl   _start
    .section .text

_start:
    la       sp, stack_top
    call     main
    mv       a0, a0        # Copy return value as is to a syscall argument register
    li       a7, 3         # Call exit syscall
    ecall
