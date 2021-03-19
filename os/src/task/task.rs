use core::cmp::{Eq, Ord, PartialOrd, Ordering};
use crate::config::BIG_STRIDE;

#[derive(Copy, Clone, Debug)]
pub struct TaskControlBlock {
    pub index: usize,
    pub task_cx_ptr: usize,
    pub task_status: TaskStatus,
    pub pass: usize,
    pub stride: usize,
    pub priority: usize
}

impl TaskControlBlock {
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize
    }

    pub fn set_priority(&mut self, pri: usize) {
        self.priority = pri;
        self.stride = BIG_STRIDE / pri;
    }

    pub fn update_pass(&mut self) {
        self.pass += self.stride;
    }
}

impl PartialEq for TaskControlBlock {
    fn eq(&self, other: &Self) -> bool {
        self.pass == other.pass && self.index == other.index
    }
}

impl Eq for TaskControlBlock {}

impl PartialOrd for TaskControlBlock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskControlBlock {
    fn cmp(&self, other: &Self) -> Ordering {
        other.pass.cmp(&self.pass)
            .then_with(|| other.priority.cmp(&self.priority))
            .then_with(|| other.index.cmp(&self.index))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited
}