.global _start
.extern boot_system

.section .phys.text, "ax"

.code32
.align 4

check_1gb:
    movl $0x80000001, %eax
    cpuid
    andl $0x04000000, %edx
    jnz  2f
1:
    hlt
    jmp  1b
2:
    ret

construct_pml4:
    /* Zero the PDPT */
    movl $phys_pdpt, %edi
    movl $0x0, %eax
    movl $1024, %ecx
1:
    movl %eax, (%edi)
    addl $4, %edi
    loop 1b
    /* Zero the PML4 */
    movl $phys_pml4, %edi
    movl $1024, %ecx
1:
    movl %eax, (%edi)
    addl $4, %edi
    loop 1b
    /* Point the first 4 entires in the PDPT to the first 4gb of memory,
     * With the first entry being duplicated in the second highest entry
     * to become our kernel window */
    movl $phys_pdpt, %edi
    movl $0x87, %eax
    movl %eax, (%edi)
    movl %eax, 4080(%edi)
    addl $0x40000000, %eax
    movl %eax, 8(%edi)
    addl $0x40000000, %eax
    movl %eax, 16(%edi)
    addl $0x40000000, %eax
    movl %eax, 24(%edi)
    /* Put the PDPT in the PML4 */
    movl $phys_pml4, %edi
    movl $phys_pdpt, %eax
    orl $0x7, %eax
    movl %eax, (%edi)
    movl %eax, 4088(%edi)
    ret

enable_long_mode:
    movl $phys_pml4, %eax
    movl %eax, %cr3
    /* Enable PAE */
    movl %cr4, %eax
    orl $0x20, %eax
    movl %eax, %cr4
    /* Set LME */
    movl $0xC0000080, %ecx
    rdmsr
    orl $0x100, %eax
    wrmsr
    /* Now turn on paging */
    movl %cr0, %eax
    orl $0x80000000, %eax
    movl %eax, %cr0
    ret

_start:
    /* Switch to temporary kernel stack to use until we can
     * set up proper kernel mappings */
    leal phys_stack_top, %esp
    movl %esp, %ebp

    /* Stash the things we got from MultiBoot
       www.gnu.org/software/grub/manual/multiboot
     */
    pushl $0
    pushl %eax /* magic (check this later) */
    pushl $0
    pushl %ebx /* info */

    /* Reset EFLAGS to disable interrupts */
    pushl $0
    popf

    /* Check we support 1GB mappings */
    call check_1gb

    /* Disable paging */
    movl %cr0, %eax
    andl $0x7fffffff, %eax
    movl %eax, %cr0

    /* Construct temporary kernel address space */
    call construct_pml4

    /* Enable paging and long mode */
    call enable_long_mode

    /* Load the gdt */
    lgdt gdt64_ptr

    /* Reload CS to enable long mode */
    ljmp $8, $_tramp_64

.code64
_tramp_64:
    /* Now in long mode and with proper virtual address space
     * we can jmp into the kernel window */
    movabs $_head64, %rax
    jmp *%rax

.section .text, "ax"
.code64

_head64:
    /* Make the stack pointer have a canonical form address */
    movq $0xffffffff80000000, %rax
    addq %rax, %rsp
    addq %rax, %rbp
    popq %rsi
    popq %rdi
    call boot_system
    /* Shouldn't return here */
1:
    hlt
    jmp 1b

.section .phys.data, "a"
gdt64_ptr:
    .word (3 * 8) - 1
    .long gdt64

.align 16
gdt64:
    .quad 0x0000000000000000
    .word   0
    .word   0
    .byte   0
    .byte   0x98
    .byte   0x20
    .byte   0
    .word   0
    .word   0
    .byte   0
    .byte   0x90
    .byte   0
    .byte   0

.align 8
phys_stack_bottom:
    .fill 16384
phys_stack_top:
.align 4096
phys_pml4:
    .fill 4096
phys_pdpt:
    .fill 4096
