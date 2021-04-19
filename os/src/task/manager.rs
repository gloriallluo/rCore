use spin::Mutex;
use lazy_static::*;
use alloc::sync::Arc;
use crate::task::task::TaskControlBlock;
use alloc::collections::vec_deque::VecDeque;

pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>
}

/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self { ready_queue: VecDeque::new() }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
    pub fn find(&self, pid: usize) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue
            .iter()
            .find(|task| task.pid.0 == pid)
            .map(|task| task.clone())
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.lock().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.lock().fetch()
}

pub fn find_task(pid: usize) -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.lock().find(pid)
}