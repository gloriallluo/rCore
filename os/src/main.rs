#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]     // 获取 panic 信息并打印

#[macro_use]
mod console;
mod lang_items;
mod sbi;

use crate::sbi::sys_exit;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main() {
    // println!("Hello World!");
    sys_exit();
}
