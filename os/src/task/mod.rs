pub mod context;
mod task;
mod switch;

use crate::config::{MAX_APP_NUM, APP_SIZE_LIMIT, BIG_STRIDE};
use crate::loader::{get_num_app, init_app_cx, get_base_i, USER_STACK};
use core::cell::{RefCell};
use core::ops::Range;
use alloc::collections::binary_heap::BinaryHeap;
use lazy_static::*;
use crate::task::switch::__switch;
use crate::task::task::{TaskControlBlock, TaskStatus};

pub struct TaskManager {
    num_app: usize,
    inner: RefCell<TaskManagerInner>
}

struct TaskManagerInner {
    tasks: BinaryHeap<TaskControlBlock>,
    current_task: Option<TaskControlBlock>
}

unsafe impl Sync for TaskManager {}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        if num_app > MAX_APP_NUM { panic!("[kernel] Too many apps!"); }
        let mut tasks: BinaryHeap<TaskControlBlock> = BinaryHeap::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock {
                index: i,
                task_cx_ptr: init_app_cx(i) as *const _ as usize,
                task_status: TaskStatus::Ready,
                pass: 0,
                stride: BIG_STRIDE / 16,
                priority: 16
            });
        }
        TaskManager {
            num_app,
            inner: RefCell::new(TaskManagerInner {
                tasks, current_task: Option::None
            })
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.current_task = Option::from(inner.tasks.pop().unwrap());
        inner.current_task.unwrap().task_status = TaskStatus::Running;
        inner.current_task.unwrap().update_pass();
        let next_task_cx_ptr2 = inner.current_task.unwrap().get_task_cx_ptr2();
        let _unused: usize = 0;
        unsafe { __switch(&_unused as *const _, next_task_cx_ptr2); }
    }

    fn get_current_task(&self) -> usize {
        let inner = self.inner.borrow();
        if let Some(current) = inner.current_task {
            current.index
        } else { panic!("no current task"); }
    }

    fn set_current_priority(&self, pri: usize) {
        let mut inner = self.inner.borrow_mut();
        if let Some(mut current) = inner.current_task {
            current.set_priority(pri)
        } else { panic!("no current task"); }
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(mut current) = inner.current_task {
            current.task_status = TaskStatus::Ready;
        } else { panic!("no current task"); }
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(mut current) = inner.current_task {
            current.task_status = TaskStatus::Exited;
        } else { panic!("no current task"); }
    }

    fn find_next_task(&self) -> Option<TaskControlBlock> {
        let inner = self.inner.borrow();
        while !inner.tasks.is_empty() {
            let task = inner.tasks.pop().unwrap();
            if task.task_status == TaskStatus::Ready {
                return Some(task);
            }
        }
        None
    }

    fn run_next_task(&self) {
        if let Some(mut next) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current = inner.current_task.unwrap();
            inner.tasks.push(current);
            next.task_status = TaskStatus::Running;
            next.update_pass();
            inner.current_task = Some(next);
            let current_task_cx_ptr2 = next.get_task_cx_ptr2();
            let next_task_cx_ptr2 = next.get_task_cx_ptr2();
            core::mem::drop(inner); // drop 掉 inner 可变引用
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

pub fn current_app_space() -> Range<usize> {
    let base = get_base_i(TASK_MANAGER.get_current_task());
    base..base + APP_SIZE_LIMIT
}

pub fn current_user_stack_top() -> usize {
    let current_task = TASK_MANAGER.get_current_task();
    USER_STACK[current_task].get_sp()
}

pub fn set_current_priority(pri: usize) {
    TASK_MANAGER.set_current_priority(pri);
}
