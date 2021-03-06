#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)] // 获取 panic 信息并打印
#![feature(alloc_error_handler)] // alloc 错误处理
#![feature(const_in_array_repeat_expressions)]

use core::env;

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod trap;
mod syscall;
mod config;
mod task;
mod timer;
mod memory;
mod fs;
mod drivers;

extern crate alloc;
extern crate bitflags;

// 将 .bss 段清零
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main() {
    clear_bss();
    println!("[kernel] Hello, world!");
    memory::init();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    fs::list_apps();
    task::add_initproc();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
