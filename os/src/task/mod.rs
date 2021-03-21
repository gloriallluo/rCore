pub mod context;
mod task;
mod switch;

use crate::config::{MAX_APP_NUM, APP_SIZE_LIMIT, BIG_STRIDE, TIME_COUNT_THRESHOLD};
use crate::loader::{get_num_app, init_app_cx, get_base_i, USER_STACK};
use core::cell::{RefCell};
use core::convert::TryInto;
use core::ops::Range;
use core::mem::drop;
use lazy_static::*;
use crate::task::switch::__switch;
use crate::task::task::{TaskControlBlock, TaskStatus};

pub struct TaskManager {
    num_app: usize,
    inner: RefCell<TaskManagerInner>
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize
}

unsafe impl Sync for TaskManager {}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [
            TaskControlBlock {
                task_cx_ptr: 0,
                task_status: TaskStatus::UnInit,
                pass: 0,
                stride: BIG_STRIDE / 16,
                priority: 16,
                count_time: 0
            }; MAX_APP_NUM
        ];
        for i in 0..num_app {
            tasks[i].task_cx_ptr = init_app_cx(i) as *const _ as usize;
            tasks[i].task_status = TaskStatus::Ready;
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
        // println!("count: {}", count);
        if count > TIME_COUNT_THRESHOLD {
            mark_current_exited();
            run_next_task();
        }
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

pub fn update_time_counter() {
    TASK_MANAGER.update_time_counter();
}
