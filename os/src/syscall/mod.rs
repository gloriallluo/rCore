#[allow(unused)]

use crate::syscall::fs::sys_write;
use crate::syscall::process::sys_exit;
use crate::trap::context::TrapContext;

mod fs;
mod process;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;

pub fn syscall(syscall_id: usize, args: [usize; 3], _cx: &TrapContext) -> isize {
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2], _cx),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id)
    }
}

