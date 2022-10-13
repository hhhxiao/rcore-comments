    .section .text.entry
    .globl _start
_start:
#设置栈指针，不然遇到函数调用就寄了，因为函数调用需要一个栈来支持
    la sp, boot_stack_top
    call rust_main

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top: