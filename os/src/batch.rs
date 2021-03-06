use core::cell::RefCell;
use core::slice::{from_raw_parts, from_raw_parts_mut};

const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x8040_0000;
const APP_SIZE_LIMIT: usize = 0x20000;

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
    static ref APP_MANAGER: AppManager {
        inner: RefCell::new({
            extern "C" { fn _num_app(); }
            let num_app_ptr = _num_app as *const usize;
            let num_app = unsafe { num_app_ptr.read_volatile() };
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            let app_start_raw: &[u8] = unsafe { from_raw_parts(num_app_ptr.add(1), num_app + 1) };
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManager { num_app, current_app, app_start }
        })
    }
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
        llvm_asm!("fence.i"::::"volatile"); // clear i cache

        // clear app area
        (APP_BASE_ADDRESS..APP_BASE_ADDRESS + APP_SIZE_LIMIT).for_each(|addr| {
            (addr as *mut u8).write_volatile(0);
        });

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
    let current_app = APP_MANAGER.inner.borrow().get_current_app();
    unsafe { APP_MANAGER.inner.borrow().load_app(current_app); }
    APP_MANAGER.inner.borrow_mut().move_to_next_app();
    extern "C" { fn __restore(cx_addr: usize); }
    // unsafe {
    //     __restore(KERNEL_STACK.push_context(
    //         TrapContext::app_init_context(APP_BASE_ADDRESS, USER_STACK.get_sp())
    //     ) as *const _ as usize);
    // }
    panic!("Unreachable in batch::run_current_app!");
}
