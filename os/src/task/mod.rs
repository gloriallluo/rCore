pub mod context;
mod task;
mod switch;

use crate::config::{TIME_COUNT_THRESHOLD};
use core::cell::{RefCell};
use core::convert::TryInto;
// use core::ops::Range;
use core::mem::drop;
use lazy_static::*;
use crate::task::switch::__switch;
use crate::task::task::{TaskControlBlock, TaskStatus};
use crate::trap::context::TrapContext;
use crate::loader::{get_num_app, get_app_data};
use alloc::vec::Vec;
use crate::memory::address::{VirtAddr};
use crate::memory::memory_set::{MapPermission};
// use crate::memory::page_table::PageTable;

pub struct TaskManager {
    num_app: usize,
    inner: RefCell<TaskManagerInner>
}

struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize
}

unsafe impl Sync for TaskManager {}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
        }
        TaskManager {
            num_app,
            inner: RefCell::new(TaskManagerInner {
                tasks, current_task: 0
            })
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.tasks[0].task_status = TaskStatus::Running;
        inner.tasks[0].update_pass();
        let next_task_cx_ptr2 = inner.tasks[0].get_task_cx_ptr2();
        let _unused: usize = 0;
        drop(inner);
        unsafe { __switch(&_unused as *const _, next_task_cx_ptr2); }
    }

    fn get_current_task(&self) -> usize {
        self.inner.borrow().current_task
    }

    fn set_current_priority(&self, pri: usize) {
        let current = self.inner.borrow().current_task;
        let mut inner = self.inner.borrow_mut();
        inner.tasks[current].set_priority(pri);
    }

    fn update_time_counter(&self) {
        let current = self.inner.borrow().current_task;
        self.inner.borrow_mut().tasks[current].count_time += 1;
        let count = self.inner.borrow().tasks[current].count_time;
        if count > TIME_COUNT_THRESHOLD {
            mark_current_exited();
            run_next_task();
        }
    }

    fn get_current_token(&self) -> usize {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        inner.tasks[current].get_user_token()
    }

    #[allow(unused)]
    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        inner.tasks[current].get_trap_cx()
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn mmap_current(&self, start_va: VirtAddr, end_va: VirtAddr,
                    permission: MapPermission) -> Option<isize> {
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        let real_start_va: VirtAddr = start_vpn.into();
        let real_end_va: VirtAddr = end_vpn.into();
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        if inner.tasks[current].memory_set.contains_vpn(start_vpn, end_vpn) {
            return None;
        }
        inner.tasks[current].memory_set.insert_framed_area(
            real_start_va, real_end_va, permission
        );
        Some((real_end_va.0 - real_start_va.0) as isize)
    }

    fn munmap_current(&self, start_va: VirtAddr, end_va: VirtAddr) -> Option<isize> {
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        if let Some(vpn_len) = inner.tasks[current].memory_set.remove_framed_area(start_vpn, end_vpn) {
            Some(vpn_len << 12)
        } else {
            None
        }
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        let mut next_task: isize = -1;
        let mut min_pass: usize = usize::max_value();
        for i in current + 1..current + self.num_app + 1 {
            let id = i % self.num_app;
            if inner.tasks[id].task_status == TaskStatus::Ready &&
                inner.tasks[id].pass < min_pass {
                next_task = id.try_into().unwrap();
                min_pass = inner.tasks[id].pass;
            }
        }
        if next_task != -1 { Some(next_task as usize) } else { None }
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current = inner.current_task;
            // TODO
            // info!("[kernel] Run task {}", next);
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.tasks[next].update_pass();
            inner.current_task = next;
            let current_task_cx_ptr2 = inner.tasks[current].get_task_cx_ptr2();
            let next_task_cx_ptr2 = inner.tasks[next].get_task_cx_ptr2();
            core::mem::drop(inner);
            unsafe { __switch(current_task_cx_ptr2, next_task_cx_ptr2); }
        } else {
            panic!("All applications completed!");
        }
    }
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

pub fn get_current_task() -> usize {
    TASK_MANAGER.get_current_task()
}

#[allow(unused)]
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

pub fn set_current_priority(pri: usize) {
    TASK_MANAGER.set_current_priority(pri);
}

pub fn update_time_counter() {
    TASK_MANAGER.update_time_counter();
}

pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

pub fn mmap_current_task(start: usize, end: usize, port: usize) -> Option<isize> {
    let mut perm: MapPermission = MapPermission::from_bits(0).unwrap();
    if port & 0x1 != 0 { perm.set(MapPermission::R, true); }
    if port & 0x2 != 0 { perm.set(MapPermission::W, true); }
    if port & 0x4 != 0 { perm.set(MapPermission::X, true); }
    perm.set(MapPermission::U, true);
    TASK_MANAGER.mmap_current(VirtAddr::from(start), VirtAddr::from(end), perm)
}

pub fn munmap_current_task(start: usize, end: usize) -> Option<isize> {
    TASK_MANAGER.munmap_current(VirtAddr::from(start), VirtAddr::from(end))
}
