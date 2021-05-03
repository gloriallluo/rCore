use alloc::sync::Arc;
use crate::fs::pipe::make_pipe;
use crate::trap::context::TrapContext;
use crate::task::processor::{current_user_token, current_task, current_pid};
use crate::memory::address::{VirtAddr, VirtPageNum};
use crate::memory::page_table::{
    PageTable,
    UserBuffer,
    translated_str,
    translated_refmut,
    translated_byte_buffer
};
use crate::fs::{File, Stat};
use crate::fs::inode::{OpenFlags, open_file, find_file_id, link_file, unlink_file};
use crate::task::manager::find_task;


fn security_check(buf: usize, len: usize) -> bool {
    let page_table = PageTable::from_token(current_user_token());
    let start_va = VirtAddr::from(buf);
    let end_va = VirtAddr::from(buf + len);
    let start_vpn = start_va.floor();
    let end_vpn = end_va.ceil();
    for vpn in start_vpn.0..end_vpn.0 {
        if let Some(pte) = page_table.translate(
            VirtPageNum::from(vpn)) {
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

    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() { return -1; }

    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release Task lock manually to avoid deadlock
        drop(inner);
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() { return -1; }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release Task lock manually to avoid deadlock
        drop(inner);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_open(_fd: usize, path: *const u8, flags: u32, _mode: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(
        path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.acquire_inner_lock();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() { return -1; }
    if inner.fd_table[fd].is_none() { return -1; }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut inner = task.acquire_inner_lock();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    // 将读端和写端的文件描述符写回到应用地址空间
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn sys_dup(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() { return -1; }
    if inner.fd_table[fd].is_none() { return -1; }
    let new_fd = inner.alloc_fd();
    inner.fd_table[new_fd] = Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
    new_fd as isize
}

pub fn sys_mail_read(buf: *mut u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if len == 0 { return if inner.mail_box_is_empty() { -1 } else { 0 }; }
    if let Some(mail) = &inner.get_mail() {
        drop(inner);
        mail.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_mail_write(pid: usize, buf: *mut u8, len: usize) -> isize {
    if !security_check(buf as usize, len) { return -1; }
    let token = current_user_token();
    let task = if current_pid() == pid {
        current_task().unwrap()
    } else {
        if let Some(t) = find_task(pid) { t } else { return -1; }
    };
    let mut inner = task.acquire_inner_lock();
    if len == 0 { return if inner.mail_box_is_full() { -1 } else { 0 }; }
    if let Some(mail) = &inner.new_empty_mail() {
        drop(inner);
        mail.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

// TODO
pub fn sys_linkat(_old_fd: i32, old_path: *const u8,
                  _new_fd: i32, new_path: *const u8,
                  _flags: u32) -> isize {
    let token = current_user_token();
    let old_path = translated_str(token, old_path);
    let new_path = translated_str(token, new_path);
    if old_path == new_path { return -1; }
    if let Some(old_id) = find_file_id(old_path.as_str()) {
        let inode = link_file(new_path.as_str(), old_id);
        return if inode.is_some() { 0 } else { -1 };
    } else {
        -1
    }
}

// TODO
pub fn sys_unlinkat(_fd: i32, path: *const u8, _flags: u32) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if unlink_file(path.as_str()) { 0 } else { -1 }
}

// TODO
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() { return -1; }
    if inner.fd_table[fd].is_none() { return -1; }
    unsafe { inner.fd_table[fd].as_ref().unwrap().stat(&mut (*st)); }
    0
}
