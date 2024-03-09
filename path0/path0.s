    .globl   _start
    .section .text

_start:
# la sp, _stack_start
    call     main
    mv       x31, a0 # Copy return value as is to a syscall argument register
    li       x30, 3  # Call exit syscall
    ecall
