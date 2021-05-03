use alloc::sync::Arc;
use core::mem::drop;

use lazy_static::*;

pub use processor::run_tasks;

use crate::fs::inode::{open_file, OpenFlags};
use crate::memory::address::VirtAddr;
use crate::memory::memory_set::MapPermission;
use crate::task::manager::add_task;
use crate::task::processor::{PROCESSOR, schedule, take_current_task};
use crate::task::task::{TaskControlBlock, TaskStatus};

pub mod context;
mod task;
mod switch;
mod pid;
pub(crate) mod manager;
pub(crate) mod processor;

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
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap(); // FIXME: unwrap None
        let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
    });
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}

pub fn mmap_current_task(start: usize, end: usize, port: usize) -> Option<isize> {
    let mut perm: MapPermission = MapPermission::from_bits(0).unwrap();
    if port & 0x1 != 0 { perm.set(MapPermission::R, true); }
    if port & 0x2 != 0 { perm.set(MapPermission::W, true); }
    if port & 0x4 != 0 { perm.set(MapPermission::X, true); }
    perm.set(MapPermission::U, true);
    PROCESSOR.mmap_current(VirtAddr::from(start), VirtAddr::from(end), perm)
}

pub fn munmap_current_task(start: usize, end: usize) -> Option<isize> {
    PROCESSOR.munmap_current(VirtAddr::from(start), VirtAddr::from(end))
}
