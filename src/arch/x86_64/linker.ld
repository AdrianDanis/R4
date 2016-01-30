ENTRY(_start)

OUTPUT_FORMAT(elf64-x86-64)

SECTIONS {
    /* We load physically to 1M */
    . = 1M;

    /* Place the header first */
    .multiboot . : {
        KEEP(*(.multiboot))
    }

    /* Place phys code/data etc here */
    .phys . : {
        *(.phys.text)
        *(.phys)
        *(.phys.*)
        *(.text)
    }

   /DISCARD/ :
    {
        *(.note.gnu.build-id)
        *(.comment)
    }
}

