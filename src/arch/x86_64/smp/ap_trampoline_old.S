/*
 * 16-bit AP trampoline — runs in real mode, switches to protected & long mode.
 * This must be identity-mapped below 1 MiB so APs can jump here after SIPI.
 */

.code16
.globl ap_trampoline_start
ap_trampoline_start:
    cli
    xor %ax, %ax
    mov %ax, %ds
    mov %ax, %ss
    lgdt gdt_descriptor
    mov %cr0, %eax
    or  $1, %eax
    mov %eax, %cr0
    jmp $0x08, $pmode_entry

.code32
pmode_entry:
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %ss
    mov %eax, %cr4
    or  $(1 << 5), %eax    /* enable PAE */
    mov %eax, %cr4
    mov $pml4_table, %eax
    mov %eax, %cr3
    mov %cr0, %eax
    or  $(1 << 31), %eax   /* enable paging */
    mov %eax, %cr0
    mov $0xC0000080, %ecx
    rdmsr
    or  $(1 << 8), %eax    /* enable long mode */
    wrmsr
    ljmp $0x28, $lmode_entry

.code64
lmode_entry:
    mov $0x30, %ax
    mov %ax, %ds
    mov %ax, %ss
    /* Pass APIC ID in %rdi */
    mov $0, %rdi
    call ap_entry64

.align 8
gdt:
    .quad 0x0000000000000000
    .quad 0x00cf9a000000ffff /* code */
    .quad 0x00cf92000000ffff /* data */
    .quad 0x00af9a000000ffff /* long mode code */
    .quad 0x00af92000000ffff /* long mode data */

gdt_descriptor:
    .word (gdt_end - gdt - 1)
    .long gdt

gdt_end:

.align 4096
pml4_table:
    .quad 0x0000000000000000

ap_trampoline_end: