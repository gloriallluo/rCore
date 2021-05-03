use bitflags::*;

pub use inode::list_apps;

use crate::memory::page_table::UserBuffer;

pub(crate) mod pipe;
pub(crate) mod stdio;
pub(crate) mod inode;
pub(crate) mod mail;

#[repr(C)]
#[derive(Debug)]
pub struct Stat {
    /// 文件所在磁盘驱动器号
    pub dev: u64,
    /// inode 文件所在 inode 编号
    pub ino: u64,
    /// 文件类型
    pub mode: StatMode,
    /// 硬链接数量，初始为1
    pub nlink: u32,
    /// 无需考虑，为了兼容性设计
    pad: [u64; 7]
}

bitflags! {
    pub struct StatMode: u32 {
        const NULL  = 0;
        /// directory
        const DIR   = 0o040000;
        /// ordinary regular file
        const FILE  = 0o100000;
    }
}

impl Stat {
    pub fn set_empty(&mut self) {
        self.dev = 0;
        self.ino = 0;
        self.nlink = 0;
        self.mode = StatMode::NULL;
        self.pad = [0u64; 7];
    }
}

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
    fn stat(&self, stat: &mut Stat);
}

