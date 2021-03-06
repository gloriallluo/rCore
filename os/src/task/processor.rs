use lazy_static::*;
use alloc::sync::Arc;
use core::cell::RefCell;
use crate::memory::address::VirtAddr;
use crate::trap::context::TrapContext;
use crate::task::switch::__switch;
use crate::task::task::{TaskControlBlock, TaskStatus};
use crate::task::manager::fetch_task;
use crate::memory::memory_set::MapPermission;


pub struct Processor {
    inner: RefCell<ProcessorInner>
}

unsafe impl Sync for Processor {}

struct ProcessorInner {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx_ptr: usize
}

impl Processor {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(ProcessorInner {
                current: None, idle_task_cx_ptr: 0
            })
        }
    }

    fn get_idle_task_cx_ptr2(&self) -> *const usize {
        let inner = self.inner.borrow();
        &inner.idle_task_cx_ptr as *const usize
    }

    pub fn run(&self) {
        loop {
            if let Some(task) = fetch_task() {
                // fetch a task from ready queue
                let idle_task_cx_ptr2 = self.get_idle_task_cx_ptr2();
                // acquire
                let mut task_inner = task.acquire_inner_lock();
                let next_task_cx_ptr2 = task_inner.get_task_cx_ptr2();
                task_inner.update_pass();
                task_inner.task_status = TaskStatus::Running;
                drop(task_inner);
                // release
                self.inner.borrow_mut().current = Some(task);
                unsafe { __switch(idle_task_cx_ptr2, next_task_cx_ptr2); }
            }
        }
    }

    /// 获得 current 的所有权
    pub fn take_current(&self) -> Option<Arc<TaskControlBlock>> {
        self.inner.borrow_mut().current.take()
    }

    /// 获得 current 的拷贝
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.inner.borrow().current.as_ref().map(|task| Arc::clone(task))
    }



    pub fn mmap_current(&self, start_va: VirtAddr, end_va: VirtAddr,
                    permission: MapPermission) -> Option<isize> {
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        let real_start_va: VirtAddr = start_vpn.into();
        let real_end_va: VirtAddr = end_vpn.into();
        let inner = self.current().unwrap();
        if inner.acquire_inner_lock().memory_set.contains_vpn(start_vpn, end_vpn) {
            return None;
        }
        inner.acquire_inner_lock().memory_set.insert_framed_area(
            real_start_va, real_end_va, permission
        );
        Some((real_end_va.0 - real_start_va.0) as isize)
    }

    pub fn munmap_current(&self, start_va: VirtAddr, end_va: VirtAddr) -> Option<isize> {
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        let inner = self.current().unwrap();
        let x = if let Some(vpn_len) = inner
            .acquire_inner_lock()
            .memory_set
            .remove_framed_area(start_vpn, end_vpn) {
            Some(vpn_len << 12)
        } else {
            None
        };
        x
    }

}

lazy_static! {
    pub static ref PROCESSOR: Processor = Processor::new();
}

pub fn run_tasks() {
    PROCESSOR.run();
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.take_current()
}

pub fn current_pid() -> usize {
    PROCESSOR.current().unwrap().getpid()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.current()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.acquire_inner_lock().get_user_token();
    token
}

pub fn set_current_priority(pri: usize) {
    let task = current_task().unwrap();
    task.acquire_inner_lock().set_priority(pri);
}

pub fn update_time_counter() {
    let task = current_task().unwrap();
    task.acquire_inner_lock().count_time += 1;
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task().unwrap().acquire_inner_lock().get_trap_cx()
}

/// 用完时间片或者调用 sys_yield 主动交出使用权
/// 切换到 idle 执行流并开启新一轮的任务调度
pub fn schedule(switched_task_cx_ptr2: *const usize) {
    let idle_task_cx_ptr2 = PROCESSOR.get_idle_task_cx_ptr2();
    unsafe { __switch(switched_task_cx_ptr2, idle_task_cx_ptr2); }
}
