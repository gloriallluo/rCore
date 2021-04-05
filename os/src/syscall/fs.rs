use crate::trap::context::TrapContext;
use crate::memory::page_table::{translated_byte_buffer, PageTable};
use crate::task::{current_user_token};
use crate::memory::address::{VirtAddr, VirtPageNum};

const FD_STDOUT: usize = 1;

fn security_check(buf: usize, len: usize) -> bool {
    let page_table = PageTable::from_token(current_user_token());
    let start_va = VirtAddr::from(buf);
    let end_va = VirtAddr::from(buf + len);
    let start_vpn = start_va.floor();
    let end_vpn = end_va.ceil();
    for vpn in start_vpn.0..end_vpn.0 {
        if let Some(pte) = page_table.translate(VirtPageNum::from(vpn)) {
            if !pte.is_valid() || !pte.readable() || !pte.u_able() { return false; }
        } else {
            return false;
        }
    }
    return true;
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize, _cx: &TrapContext) -> isize {
    // security check
    if !security_check(buf as usize, len) { return -1; }

    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        },
        _ => {
            warn!("Unsupported fd: {} in sys_write!", fd);
            -1 as isize
        }
    }
}