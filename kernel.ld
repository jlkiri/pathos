OUTPUT_ARCH("riscv")

ENTRY(_start)

MEMORY {
  ram (wxa) : ORIGIN = 0x80000000, LENGTH = 128M
}

PHDRS
{
  text PT_LOAD;
  data PT_LOAD;
  bss PT_LOAD;
}

SECTIONS {
  .text : {
    PROVIDE(_text_start = .);
    *(.text.boot) *(.text .text.*)
    PROVIDE(_text_end = .);
  } > ram :text

  .rodata : {
    PROVIDE(_rodata_start = .);
    *(.rodata .rodata.*)
    PROVIDE(_rodata_end = .);
  } > ram :text

  .data : {
    PROVIDE(_data_start = .);
    *(.sdata .sdata.*) *(.data .data.*)
    PROVIDE(_data_end = .);
  } > ram :data
  
  .bss : {
    PROVIDE(_bss_start = .);
    *(.sbss .sbss.*) *(.bss .bss.*)
    PROVIDE(_bss_end = .);
  } > ram :bss

  PROVIDE(_memory_start = ORIGIN(ram));
  PROVIDE(_memory_end = ORIGIN(ram) + LENGTH(ram));

  PROVIDE(_stack_start = _bss_end);
  PROVIDE(_stack_end = _stack_start + 1M);
  
  PROVIDE(_heap_start = _stack_end);
  PROVIDE(_heap_size = 4M);

  . = _heap_start + _heap_size;
  . = ALIGN(4096 * 4);
  PROVIDE(_alloc_start = .);
  PROVIDE(_alloc_size  = 4M);
}