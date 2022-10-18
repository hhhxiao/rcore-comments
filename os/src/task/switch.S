.altmacro
.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)
.endm
    .section .text
    .globl __switch

# User call yield -> __alltraps -> trap_handler -> __switch --> __restore
# User call的上下文在trapcontext中，然后__allrtaps执行的信息和trapCtx一样也在内核栈中
# 因此当前的内核栈有如下几个内容 
# TrapCtx 
# trap的执行栈信息
# 如果这个时候需要切换控制流的化，就需要保存这些信息，然后切恢复另一个控制流的相关信息
# TrapCtx + 执行栈 ==> 保存当前应用的内核栈指针即可
# 函数调用信息 -> 保存部分寄存器即可
#

# 这里需要注意的是，每个应用都有自己的内核栈，我们切换了内核栈就相当于切换了当前执行的上下文


__switch:
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext,
    #     next_task_cx_ptr: *const TaskContext
    # )
    # save kernel stack of current task
    sd sp, 8(a0)
    # save ra & s0~s11 of current execution
    sd ra, 0(a0)
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr
    # restore ra & s0~s11 of next execution
    ld ra, 0(a1)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    # restore kernel stack of next task
    # 切换下一个应用的内核栈
    ld sp, 8(a1)
    ret

