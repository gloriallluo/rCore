pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const MAX_APP_NUM: usize = 16;
pub const APP_BASE_ADDRESS: usize = 0x8040_0000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
pub const CLOCK_FREQ: usize = 1250_0000;
pub const BIG_STRIDE: usize = 0x1000_0000;

// config for k-210
// pub const CLOCK_FREQ: usize = 403000000 / 62;
