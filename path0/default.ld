OUTPUT_ARCH("riscv")

ENTRY(_start)

MEMORY {
    mem : ORIGIN = 0x80000000, LENGTH = 0x1000
}

SECTIONS {
    .text : {
        . = 0xc00000000;
        *(.text)
    }
}