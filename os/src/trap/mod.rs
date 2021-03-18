use riscv::register::{
    mtvec::TrapMode, stvec, scause::{self, Trap, Interrupt, Exception}, stval, sie
};
use crate::trap::context::TrapContext;
use crate::task::{
    exit_current_and_run_next, suspend_current_and_run_next
};
use crate::syscall::syscall;
use crate::timer::set_next_trigger;

pub mod context;

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" { fn __alltraps(); }
    unsafe { stvec::write(__alltraps as usize, TrapMode::Direct); }
}

pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer(); }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4; // execute the next instruction after return
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]], cx) as usize;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] Store Page Fault in application, core dumped.");
            exit_current_and_run_next();
        },
        Trap::Exception(Exception::LoadFault) |
        Trap::Exception(Exception::LoadPageFault) => {
            println!("[kernel] Load Page Fault in application, core dumped.");
            exit_current_and_run_next();
        },
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            exit_current_and_run_next();
        },
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        },
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    cx
}