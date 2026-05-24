OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(kmain)

SECTIONS {
    . = 0xffffffff80000000;

    .text : ALIGN(0x1000) {
        *(.text .text.*)
    }

    .rodata : ALIGN(0x1000) {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(0x1000) {
        *(.data .data.*)
        *(.limine_reqs_start)
        *(.limine_reqs .limine_reqs.*)
        *(.limine_reqs_end)
    }

    .bss : ALIGN(0x1000) {
        *(COMMON)
        *(.bss .bss.*)
        . = ALIGN(0x1000);
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
        *(.got)
        *(.got.plt)
        *(.dynamic)
        *(.dynsym)
        *(.dynstr)
        *(.rela*)
    }
}