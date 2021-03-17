#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]     // 获取 panic 信息并打印
#![feature(const_in_array_repeat_expressions)]

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod trap;
mod syscall;
mod loader;
mod config;
mod task;

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
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub extern "C" fn rust_main() {
    clear_bss();
    println!("[kernel] Hello, world!");
    trap::init();
}
