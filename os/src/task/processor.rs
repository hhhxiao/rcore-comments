//!Implementation of [`Processor`] and Intersection of control flow
use super::__switch;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;
///Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    /// 当前执行的任务的控制块
    current: Option<Arc<TaskControlBlock>>,
    ///The basic control flow of each core, helping to select and switch process
    /// CPU当前执行的上下文
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(), //初始化的时候初始化一个假的出来，后面直接切换就行
        }
    }
    ///Get mutable reference to `idle_task_cx`
    /// 获取当前正在执行的任务的上下文
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
    ///Get current task in moving semanteme
    ///获取当前正在执行的任务控制块
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    ///Get current task in cloning semanteme

    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

//PROCSSOR 是全局的CPU状态管理器对象
lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}
///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();

        if let Some(task) = fetch_task() {
            //获取当前任务上下文（如果是第一次执行那么这里就是空的）
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            //获取需要切换的task的上下文指针
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            //修改状态为runninss
            task_inner.task_status = TaskStatus::Running;
            drop(task_inner);
            // release coming task TCB manually
            //手动修改CPU状态
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                //通过控制流切换到新的任务
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        }
    }
}
///Take the current task,leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}
///Get running task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}
///Get token of the address space of current task
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.inner_exclusive_access().get_user_token();
    token
}
///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}
///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
