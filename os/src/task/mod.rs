pub mod context;
mod task;
mod switch;
mod pid;
pub(crate) mod manager;
pub(crate) mod processor;

use core::mem::drop;
use lazy_static::*;
use alloc::sync::Arc;
use crate::task::manager::add_task;
use crate::task::task::{TaskControlBlock, TaskStatus};
use crate::task::processor::{take_current_task, schedule};
use crate::loader::get_app_data_by_name;


pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- hold current PCB lock
    let mut task_inner = task.acquire_inner_lock();
    let task_cx_ptr2 = task_inner.get_task_cx_ptr2();
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB lock

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr2);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();
    // **** hold current PCB lock
    let mut inner = task.acquire_inner_lock();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    {
        // ++++++ hold initproc PCB lock here
        let mut initproc_inner = INITPROC.acquire_inner_lock();
        for child in inner.children.iter() {
            child.acquire_inner_lock().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
        // ++++++ release parent PCB lock here
    }

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // **** release current PCB lock
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let _unused: usize = 0;
    schedule(&_unused as *const _);
}

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(
        TaskControlBlock::new(get_app_data_by_name("initproc").unwrap())
    );
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}

pub fn mmap_current_task(start: usize, end: usize, _port: usize) -> Option<isize> {
    // let mut perm: MapPermission = MapPermission::from_bits(0).unwrap();
    // if port & 0x1 != 0 { perm.set(MapPermission::R, true); }
    // if port & 0x2 != 0 { perm.set(MapPermission::W, true); }
    // if port & 0x4 != 0 { perm.set(MapPermission::X, true); }
    // perm.set(MapPermission::U, true);
    // TASK_MANAGER.mmap_current(VirtAddr::from(start), VirtAddr::from(end), perm)
    Some((end - start) as isize)
}

pub fn munmap_current_task(start: usize, end: usize) -> Option<isize> {
    // TASK_MANAGER.munmap_current(VirtAddr::from(start), VirtAddr::from(end))
    Some((end - start) as isize)
}
