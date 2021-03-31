pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const MEMORY_END: usize = 0x8080_0000;

pub const MAX_APP_NUM: usize = 16;
pub const APP_BASE_ADDRESS: usize = 0x8040_0000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
pub const BIG_STRIDE: usize = 0xc0000;

pub const CLOCK_FREQ: usize = 1250_0000;
pub const TIME_COUNT_THRESHOLD: usize = 2000;
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

// config for k-210
// pub const CLOCK_FREQ: usize = 403000000 / 62;
