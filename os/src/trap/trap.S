.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2

#陷入指令
__alltraps:
    #在执行这个指令之前sp->用户栈 sscratch指向内核栈
    csrrw sp, sscratch, sp
    #上面这条指令的意思就是交换，也就是sp指向内核栈，而ssc指向用户栈了
    # now sp->kernel stack, sscratch->user stack
    # allocate a TrapContext on kernel stack
    #陷入之前把寄存器数据存入内核栈
    addi sp, sp, -34*8
    # save general-purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it on the kernel stack
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # set input argument of trap_handler(cx: &mut TrapContext)
    # 存完成后把trapcontext作为指针传给trap_hander
    mv a0, sp
    call trap_handler

__restore:
    #我们首先看case1,a0是传入的参数，也就是内核栈指针
    # sp 指向内核栈
    mv sp, a0
    #如果这里是第一次执行__restore,sscrath还没初始化，然后sp指向内核栈
    #注意到我们在初始化时往栈内推入了一个默认上下文，这叫相当于强制让os处于trap状态
    #然后下面再恢复相关内容
    ld t0, 32*8(sp) #读取栈数据(sstatus)
    ld t1, 33*8(sp) #读取栈数据(sepc)
    ld t2, 2*8(sp) #读取栈数据(t2)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2 //sscratch初始化为用户栈
    #cas1继续往后，从栈中读取寄存器信息，然后写入寄存器(初始化寄存器以执行用户程序)
    # restore general-purpuse registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    #case1 恢复栈指针
    addi sp, sp, 34*8
    #到目前位置 sp-> 内核栈 sscratch指向用户栈
    #执行先下面这条指令后，sp->就变成用户栈，sscratch就变成内核栈了
    csrrw sp, sscratch, sp
    sret

