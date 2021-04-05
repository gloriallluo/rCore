use crate::task::{
    exit_current_and_run_next,
    suspend_current_and_run_next,
    set_current_priority,
    get_current_task,
    current_user_token
};
use crate::timer::{
    get_time_val,
    TimeVal
};
use crate::memory::page_table::PageTable;
use crate::memory::address::VirtAddr;

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application {} exited with code {}", get_current_task(), exit_code);
    exit_current_and_run_next();
    panic!("Application {}: Unreachable in sys_exit!", get_current_task());
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time(ts: usize, _tz: usize) -> isize {
    let page_table = PageTable::from_token(current_user_token());
    let start_va = VirtAddr::from(ts);
    let vpn = start_va.floor();
    let ppn = page_table.translate(vpn).unwrap().ppn();
    let time_val = (ppn.0 << 12 | start_va.page_offset() as usize) as *mut TimeVal;
    unsafe { (*time_val) = get_time_val(); }
    0
}

pub fn sys_set_priority(pri: isize) -> isize {
    if pri >= 2 && pri <= isize::MAX {
        set_current_priority(pri as usize);
        pri
    } else {
        -1 as isize
    }
}
