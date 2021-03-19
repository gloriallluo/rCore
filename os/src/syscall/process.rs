use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, set_current_priority};
use crate::timer::{TimeVal, get_time_val};

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time(ts: usize, _tz: usize) -> isize {
    let time_val = ts as *mut TimeVal;
    unsafe { (*time_val) = get_time_val(); }
    0
}

pub fn sys_set_priority(pri: isize) -> isize {
    if pri >= 2 && pri <= isize::MAX {
        set_current_priority(pri as usize);
        pri
    } else { -1 as isize }
}