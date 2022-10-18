//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
/// 任务管理器，或者说叫调度器，这里使用一个双端队列，两边都能进出
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    ///初始化就是创建一个新的任务队列
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    ///Add a task to `TaskManager`
    /// 添加一个新任务
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    ///Remove the first task and return it,or `None` if `TaskManager` is empty
    /// 从队列中取出一个任务用于执行
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}
///下面是全局实例对外提供的一个接口
///Interface offered to add task
pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}
///Interface offered to pop the first task
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}
