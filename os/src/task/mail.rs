use crate::fs::File;
use crate::memory::page_table::UserBuffer;
use alloc::sync::Arc;
use spin::Mutex;

const MAX_MAIL_LENGTH: usize = 256;

pub struct MailBuffer {
    arr: [u8; MAX_MAIL_LENGTH],
    length: usize
}

impl MailBuffer {
    pub fn new() -> Self {
        Self { arr: [0u8; MAX_MAIL_LENGTH], length: 0 }
    }
    // 读到 UserBuffer 里面去
    pub fn read_all(&self, buf: UserBuffer) -> usize {
        if self.length == 0 { return 0; }
        let mut len = buf.len();
        if len > self.length { len = self.length; }
        let mut buf_iter = buf.into_iter();
        for i in 0..len {
            if let Some(byte_ref) = buf_iter.next() {
                unsafe { *byte_ref = self.arr[i]; }
            } else {
                return (i + 1) as usize;
            }
        }
        len
    }
    // 从 UserBuffer 写到自己
    pub fn write_all(&mut self, buf: UserBuffer) -> usize {
        let mut buf_iter = buf.into_iter();
        self.length = 0;
        loop {
            if self.length == MAX_MAIL_LENGTH {
                return self.length
            } else if let Some(byte_ref) = buf_iter.next() {
                self.arr[self.length] = unsafe { *byte_ref };
                self.length += 1;
            } else {
                return self.length;
            }
        }
    }
}

pub struct Mail {
    buffer: Arc<Mutex<MailBuffer>>
}

impl Mail {
    pub fn new() -> Self {
        Self { buffer: Arc::new(Mutex::new(MailBuffer::new())) }
    }
}

impl File for Mail {
    fn readable(&self) -> bool { true }
    fn writable(&self) -> bool { true }
    fn read(&self, buf: UserBuffer) -> usize {
        self.buffer.lock().read_all(buf)
    }
    // 从 UserBuffer 写入 Mail
    fn write(&self, buf: UserBuffer) -> usize {
        self.buffer.lock().write_all(buf)
    }
}
