//! Implementation of [`TrapContext`]

use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
/// trap context structure containing sstatus, sepc and registers
pub struct TrapContext {
    /// general regs[0..31]
    pub x: [usize; 32], //和ch2一样还是寄存器
    /// CSR sstatus      
    pub sstatus: Sstatus, //状态寄存器
    /// CSR sepc
    pub sepc: usize, //回调地址
    /// Addr of Page Table
    pub kernel_satp: usize, //内核地址空间的satp寄存器(可以理解为内核地址空间的句柄)
    /// kernel stack
    pub kernel_sp: usize, //当前进程的内核栈栈顶指针
    /// Addr of trap_handler function
    pub trap_handler: usize, //trap_handler的虚拟地址(由于用户无法直接访问__trap_handler)
}

impl TrapContext {
    /// set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    /// init app context
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read(); // CSR sstatus
        sstatus.set_spp(SPP::User); //previous privilege mode: user mode
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,  // entry point of app
            kernel_satp,  // addr of page table
            kernel_sp,    // kernel stack
            trap_handler, // addr of trap_handler function
        };
        cx.set_sp(sp); // app's user stack pointer
        cx // return initial Trap Context of app
    }
}
