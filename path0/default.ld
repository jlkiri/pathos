OUTPUT_ARCH("riscv")

ENTRY(_start)

MEMORY {
    ram (wxa) : ORIGIN = 0x400000000000, LENGTH = 4M
}

SECTIONS {
    .text : {
        PROVIDE(_text_start = .);
        *(.text .text.*)
        PROVIDE(_text_end = .);
    } > ram

    .rodata : {
        PROVIDE(_rodata_start = .);
        *(.rodata .rodata.*)
        PROVIDE(_rodata_end = .);
    } > ram

    .data : {
        PROVIDE(_data_start = .);
        *(.sdata .sdata.*) *(.data .data.*)
        PROVIDE(_data_end = .);
    } > ram
    
    .bss : {
        PROVIDE(_bss_start = .);
        *(.sbss .sbss.*) *(.bss .bss.*)
        PROVIDE(_bss_end = .);
    } > ram

    PROVIDE(_memory_start = ORIGIN(ram));
    PROVIDE(_memory_end = ORIGIN(ram) + LENGTH(ram));

    PROVIDE(_stack_start = _bss_end);
    PROVIDE(_stack_end = _stack_start + 1M);

    . = _stack_end;
    . = ALIGN(4096);
    
    PROVIDE(_heap_start = .);
    PROVIDE(_heap_size = 2M);
}