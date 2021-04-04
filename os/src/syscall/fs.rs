use crate::trap::context::TrapContext;
use crate::memory::page_table::translated_byte_buffer;
use crate::task::{current_user_token};

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize, _cx: &TrapContext) -> isize {
    // // security check
    // let app_range = current_app_space();
    // let in_app_range = app_range.contains(&(buf as usize)) &&
    //     app_range.contains(&(buf as usize + len));
    // let stack_range = cx.x[2]..current_user_stack_top();
    // let in_stack_range = stack_range.contains(&(buf as usize)) &&
    //     stack_range.contains(&(buf as usize + len));
    // if (!in_app_range) && (!in_stack_range) {
    //     return -1 as isize;
    // }

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