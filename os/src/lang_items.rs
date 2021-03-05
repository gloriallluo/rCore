use core::panic::PanicInfo;
use crate::sbi::sys_exit;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!("Panic at {}:{}\n\t{}",
                 location.file(),
                 location.line(),
                 info.message().unwrap());
    } else {
        error!("Panic!\n\t{}",
                 info.message().unwrap());
    }
    sys_exit()
}