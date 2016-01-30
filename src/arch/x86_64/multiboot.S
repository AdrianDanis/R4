.section .multiboot, "a"
.align 4
.long 0x1BADB002
.long 3
.long - 0x1BADB002 - 3
