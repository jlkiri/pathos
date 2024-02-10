    .global  _start
    .extern  _STACK_PTR

    .section .text

_start:
    la       sp, _STACK_PTR
    call     main
    j        .