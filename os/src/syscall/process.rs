//! Process management syscalls
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::get_time_ms;

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    // exit也是让出当前的CPU控制权，分成两个部分，1，修改当前状态为**执行结束**，2切换到下一个程序执行
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
/// yield 让出当前的CPU控制权，分成两个部分，1，修改当前状态为等待，2切换到下一个程序执行
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

/// get time in milliseconds
pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}
