use crate::fs::pipe::make_pipe;
use crate::trap::context::TrapContext;
use crate::task::processor::{current_user_token, current_task, current_pid};
use crate::memory::address::{VirtAddr, VirtPageNum};
use crate::memory::page_table::{
    PageTable,
    UserBuffer,
    translated_refmut,
    translated_byte_buffer
};
use crate::fs::File;
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
