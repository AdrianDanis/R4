.global _start

.section .phys.text

.code32

_start:
    jmp _start
    /* Insert a temporary long jump to 64 bit code. We need
     * some 64-bit code to produce a valid 64bit elf, and
     * without this --gc-sections aggressively removes
     * all other 64-bit code */
    ljmp $8, $func

.section .text
.code64
func:
    jmp func
