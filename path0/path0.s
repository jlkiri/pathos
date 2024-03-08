    .globl   _start
    .section .text

_start:
    la       sp, 0xc00000000
    call     main
    mv       a0, a0          # Copy return value as is to a syscall argument register
    li       a7, 3           # Call exit syscall
    ecall
