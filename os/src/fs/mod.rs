pub(crate) mod pipe;
pub(crate) mod stdio;
pub(crate) mod inode;

use crate::memory::page_table::UserBuffer;
pub use inode::list_apps;

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

