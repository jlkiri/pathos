    .global  _start
    .extern  _STACK_PTR

    .section .text

_start:
    mv       zero, a0
    la       sp, _STACK_PTR
    call     main
    j        .