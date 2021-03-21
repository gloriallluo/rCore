use crate::config::BIG_STRIDE;

#[derive(Copy, Clone, Debug)]
pub struct TaskControlBlock {
    pub task_cx_ptr: usize,
    pub task_status: TaskStatus,
    pub pass: usize,
    pub stride: usize,
    pub priority: usize,
    pub count_time: usize
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
    UnInit
}