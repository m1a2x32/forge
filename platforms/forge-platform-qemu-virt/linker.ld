OUTPUT_ARCH(aarch64)
ENTRY(_start)

MEMORY { RAM (rwx) : ORIGIN = 0x40080000, LENGTH = 127M + 512K }
PHDRS {
    text PT_LOAD FLAGS(5);
    rodata PT_LOAD FLAGS(4);
    data PT_LOAD FLAGS(6);
}
SECTIONS {
    . = ORIGIN(RAM);
    .text : {
        KEEP(*(.text.boot))
        KEEP(*(.text.vectors))
        *(.text .text.*)
    } > RAM :text
    .rodata ALIGN(4096) : { *(.rodata .rodata.*) } > RAM :rodata
    .data ALIGN(4096) : { *(.data .data.*) *(.got .got.*) } > RAM :data
    .bss (NOLOAD) : ALIGN(8) {
        __bss_start = .;
        *(.bss .bss.*) *(COMMON)
        . = ALIGN(8);
        __bss_end = .;
    } > RAM :data
    .boot_pages (NOLOAD) : ALIGN(4096) {
        __boot_pages_start = .;
        . += 64K;
        __boot_pages_end = .;
    } > RAM :data
    .stack (NOLOAD) : ALIGN(4096) {
        __stack_bottom = .;
        . += 64K;
        __stack_top = .;
    } > RAM :data
    __kernel_end = .;
    /DISCARD/ : { *(.comment) *(.eh_frame*) *(.note*) }
}
ASSERT((_start == ORIGIN(RAM)), "Image header must be first")
ASSERT((__exception_vectors % 2048 == 0), "Vectors must be 2 KiB aligned")
ASSERT((__stack_top % 16 == 0), "AArch64 stack must be 16-byte aligned")
ASSERT((__kernel_end < 0x40200000), "Bootstrap image exceeded its 2 MiB budget")
