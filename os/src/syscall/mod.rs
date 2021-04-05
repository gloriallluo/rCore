#[allow(unused)]

use crate::syscall::fs::*;
use crate::syscall::mem::*;
use crate::syscall::process::*;
use crate::trap::context::TrapContext;

mod fs;
mod process;
mod mem;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_SET_PRIORITY: usize = 140;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_MUNMAP: usize = 215;
const SYSCALL_MMAP: usize = 222;

pub fn syscall(syscall_id: usize, args: [usize; 3], _cx: &TrapContext) -> isize {
    // println!("in syscall, syscall id {}", syscall_id);
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2], _cx),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0], args[1]),
        SYSCALL_SET_PRIORITY => sys_set_priority(args[0] as isize),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id)
    }
}

