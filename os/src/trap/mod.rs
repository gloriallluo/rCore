use riscv::register::{
    mtvec::TrapMode, stvec, stval, sie,
    scause::{self, Trap, Interrupt, Exception}
};
use crate::task::{
    exit_current_and_run_next, suspend_current_and_run_next, update_time_counter,
    current_user_token, current_trap_cx
};
use crate::syscall::syscall;
use crate::timer::set_next_trigger;
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};

pub mod context;

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" { fn __alltraps(); }
    unsafe { stvec::write(__alltraps as usize, TrapMode::Direct); }
}

#[allow(unused)]
fn set_kernel_trap_entry() {
    unsafe { stvec::write(trap_from_kernel as usize, TrapMode::Direct); }
}

fn set_user_trap_entry() {
    unsafe { stvec::write(TRAMPOLINE as usize, TrapMode::Direct); }
}

pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer(); }
}

#[no_mangle]
pub fn trap_handler() -> ! {
    let cx = current_trap_cx();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4; // execute the next instruction after return
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]], cx) as usize;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] Store Page Fault in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                     stval, cx.sepc);
            exit_current_and_run_next();
        },
        Trap::Exception(Exception::LoadFault) |
        Trap::Exception(Exception::LoadPageFault) => {
            println!("[kernel] Load Page Fault in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                     stval, cx.sepc);
            exit_current_and_run_next();
        },
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] Illegal Instruction in application, bad instruction = {:#x}, core dumped.",
                     stval);
            exit_current_and_run_next();
        },
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            update_time_counter();
            set_next_trigger();
            suspend_current_and_run_next();
        },
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    trap_return()
}

#[no_mangle]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        llvm_asm!("fence.i" :::: "volatile");
        llvm_asm!("jr $0" :: "r"(restore_va), "{a0}"(trap_cx_ptr), "{a1}"(user_satp) :: "volatile");
    }
    panic!("Unreachable in back_to_user!");
}

#[no_mangle]
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}
