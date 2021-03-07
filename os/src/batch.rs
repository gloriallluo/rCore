use lazy_static::*;
use core::cell::RefCell;
use core::mem::size_of;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use crate::trap::context::TrapContext;

const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x8040_0000;
const APP_SIZE_LIMIT: usize = 0x20000;
const USER_STACK_SIZE: usize = 8 * 1024;
const KERNEL_STACK_SIZE: usize = 8 * 1024;

struct AppManagerInner {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1]
}

struct AppManager {
    inner: RefCell<AppManagerInner>
}

unsafe impl Sync for AppManager {}

lazy_static! {
    static ref APP_MANAGER: AppManager = AppManager {
        inner: RefCell::new({
            extern "C" { fn _num_app(); }
            let num_app_ptr = _num_app as *const usize;
            let num_app = unsafe { num_app_ptr.read_volatile() };
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            let app_start_raw: &[usize] = unsafe { from_raw_parts(num_app_ptr.add(1), num_app + 1) };
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManagerInner { num_app, current_app: 0, app_start }
        })
    };
}

impl AppManagerInner {
    pub fn print_app_info(&self) {
        info!("[kernel] num_app: {}", self.num_app);
        for i in 0..self.num_app {
            info!("[kernel] app {}: [{}, {}]",
                  i, self.app_start[i], self.app_start[i + 1]);
        }
    }
    pub fn get_current_app(&self) -> usize {
        self.current_app
    }
    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            panic!("[kernel] All apps completed!");
        }
        info!("loading app {}...", app_id);

        // clear i cache
        llvm_asm!("fence.i"::::"volatile");

        // clear app area
        (APP_BASE_ADDRESS..APP_BASE_ADDRESS + APP_SIZE_LIMIT).for_each(|addr| {
            (addr as *mut u8).write_volatile(0);
        });

        // move app from `app_src` to `app_dst`
        let app_src = from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id]
        );
        let app_dst = from_raw_parts_mut(
            APP_BASE_ADDRESS as *mut u8,
            app_src.len()
        );
        app_dst.copy_from_slice(app_src);
    }
}

pub fn init() {
    print_app_info();
}

pub fn print_app_info() {
    APP_MANAGER.inner.borrow().print_app_info();
}

pub fn run_next_app() -> ! {
    // load new app into user memory (8040_0000) and increase the app ptr
    let current_app = APP_MANAGER.inner.borrow().get_current_app();
    unsafe { APP_MANAGER.inner.borrow().load_app(current_app); }
    APP_MANAGER.inner.borrow_mut().move_to_next_app();

    // run `__restore`
    extern "C" { fn __restore(cx_addr: usize); }
    unsafe {
        __restore(KERNEL_STACK.push_context(
            // change to U-Mode
            TrapContext::app_init_context(APP_BASE_ADDRESS, USER_STACK.get_sp())
        ) as *const _ as usize);
    }
    panic!("Unreachable in batch::run_current_app!");
}

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE]
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE]
}

impl KernelStack {
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    /// move `cx` into stack
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - size_of::<TrapContext>()) as *mut TrapContext;
        unsafe { *cx_ptr = cx; }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    pub fn get_sp(&self) -> usize { self.data.as_ptr() as usize + USER_STACK_SIZE }
}

static KERNEL_STACK: KernelStack = KernelStack { data: [0; KERNEL_STACK_SIZE] };
static USER_STACK: UserStack = UserStack { data: [0; USER_STACK_SIZE] };
