use crate::config::{BIG_STRIDE, TRAP_CONTEXT, kernel_stack_position};
use crate::memory::memory_set::{MemorySet, MapPermission, KERNEL_SPACE};
use crate::memory::address::{PhysPageNum, VirtAddr};
use crate::task::context::TaskContext;
use crate::trap::context::TrapContext;
use crate::trap::trap_handler;

#[derive(Debug)]
pub struct TaskControlBlock {
    pub task_cx_ptr: usize,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub pass: usize,
    pub stride: usize,
    pub priority: usize,
    pub count_time: usize
}

impl TaskControlBlock {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;
        // map a kernel-stack in kernel space
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE
            .lock()
            .insert_framed_area(
                kernel_stack_bottom.into(),
                kernel_stack_top.into(),
                MapPermission::R | MapPermission::W,
            );
        let task_cx_ptr = (kernel_stack_top - core::mem::size_of::<TaskContext>()) as *mut TaskContext;
        unsafe { *task_cx_ptr = TaskContext::goto_trap_return(); }
        let task_control_block = Self {
            task_cx_ptr: task_cx_ptr as usize,
            task_status, memory_set, trap_cx_ppn,
            base_size: user_sp, pass: 0,
            stride: BIG_STRIDE / 16, priority: 16, count_time: 0
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            kernel_stack_top,
            trap_handler as usize
        );
        task_control_block
    }

    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
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
    Exited
}