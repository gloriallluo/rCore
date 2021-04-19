use riscv::register::time;
use crate::config::CLOCK_FREQ;
use crate::sbi::sbi_set_timer;

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;
const USEC_PER_SEC: usize = 1000_000;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize
}

pub fn get_time() -> usize {
    time::read()
}

#[allow(unused)]
pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
}

#[allow(unused)]
pub fn get_time_val() -> TimeVal {
    let time = get_time();
    TimeVal {
        sec: time / CLOCK_FREQ,
        usec: (time % CLOCK_FREQ) * USEC_PER_SEC / CLOCK_FREQ
    }
}

pub fn set_next_trigger() {
    sbi_set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
