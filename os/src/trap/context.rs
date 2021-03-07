use riscv::register::sstatus::{self, Sstatus, SPP};

pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize
    // scause / stval: used / preserved by hardware at the very beginning
    // stvec: will not be changed
    // scratch: user sp
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32], sstatus, sepc: entry,
        };
        cx.set_sp(sp);
        cx
    }
}