use crate::task::{mmap_current_task, munmap_current_task};

/// 申请内存
/// port: 0|X|W|R
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    if port & 0x7 == 0 { return -1; } // pte
    if port & 0x8 != 0 { return -1; } // port[3] must be 0
    if start & 0xfff != 0 { return -1; } // 4096 的整数倍
    if let Some(length) = mmap_current_task(start, start + len, port) {
        length
    } else {
        -1
    }
}

/// 申请释放内存
pub fn sys_munmap(start: usize, len: usize) -> isize {
    if start & 0xfff != 0 { return -1; } // 4096 的整数倍
    if let Some(length) = munmap_current_task(start, start + len) {
        length
    } else {
        -1
    }
}