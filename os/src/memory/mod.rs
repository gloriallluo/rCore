pub(crate) mod address;
pub(crate) mod page_table;
pub(crate) mod frame_allocator;
mod heap_allocator;
pub(crate) mod memory_set;

use crate::memory::memory_set::KERNEL_SPACE;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.lock().activate(); // KERNEL_SPACE 被初始化
}