use alloc::sync::Arc;
use crate::task::{
    exit_current_and_run_next,
    suspend_current_and_run_next
};
use crate::task::manager::add_task;
use crate::task::processor::{
    current_task,
    current_pid,
    current_user_token,
    set_current_priority
};
use crate::timer::{
    get_time_val,
    TimeVal
};
use crate::memory::page_table::{
    PageTable,
    translated_str,
    translated_refmut
};
use crate::memory::address::VirtAddr;
use crate::loader::get_app_data_by_name;

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
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

pub fn sys_getpid() -> isize {
    current_pid() as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.acquire_inner_lock().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

pub fn sys_spawn(path: *const u8) -> isize {
    let token = current_user_token();
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    let path = translated_str(token, path);
    let trap_cx = new_task.acquire_inner_lock().get_trap_cx();
    trap_cx.x[10] = 0;
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        new_task.exec(data);
        let trap_cx = new_task.acquire_inner_lock().get_trap_cx();
        trap_cx.x[10] = 0;
        add_task(new_task);
        new_pid as isize
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    // find a child process

    // ---- hold current PCB lock
    let mut inner = task.acquire_inner_lock();
    if inner.children
        .iter()
        .find(|p| {pid == -1 || pid as usize == p.getpid()})
        .is_none() {
        return -1;
        // ---- release current PCB lock
    }
    let pair = inner.children
        .iter()
        .enumerate()
        .find(|(_, p)| {
            // ++++ temporarily hold child PCB lock
            p.acquire_inner_lock().is_zombie() && (pid == -1 || pid as usize == p.getpid())
            // ++++ release child PCB lock
        });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after removing from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily hold child lock
        let exit_code = child.acquire_inner_lock().exit_code;
        // ++++ release child PCB lock
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB lock automatically
}
