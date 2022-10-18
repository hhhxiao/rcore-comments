//! Types related to task management

use super::TaskContext;

#[derive(Copy, Clone)]

///任务控制块
///
pub struct TaskControlBlock {
    pub task_status: TaskStatus, //每个任务的状态
    pub task_cx: TaskContext,    //每个任务的上下文
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
