use core::panic::PanicInfo;
use crate::sbi::sys_exit;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("Panic!");
    sys_exit()
}