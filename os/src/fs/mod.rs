pub(crate) mod pipe;
pub(crate) mod stdio;

use crate::memory::page_table::UserBuffer;

pub trait File: Send + Sync {
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

