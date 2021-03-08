#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]     // 获取 panic 信息并打印

use core::env;

#[macro_use]
mod console;
mod lang_items;
mod sbi;

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
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn boot_stack();
        fn boot_stack_top();
    }
    clear_bss();

    let log: &'static str = env!("LOG");
    if log == "INFO" {
        info!("Hello, world!");
        info!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        info!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        info!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        info!("boot_stack [{:#x}, {:#x})", boot_stack as usize, boot_stack_top as usize);
        info!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    } else {
        trace!("Hello, world!");
        info!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        debug!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        warn!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        error!("boot_stack [{:#x}, {:#x})", boot_stack as usize, boot_stack_top as usize);
        println!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    }
    panic!("Shutdown machine!");
}
