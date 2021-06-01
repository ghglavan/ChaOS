use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

use crate::task;

pub trait Os {
    fn reset_timer(&mut self);
    fn get_switch_pair(&mut self) -> (&mut task::Task, &task::Task);
    fn get_initial_task_regs(&self) -> (*const u32, u32, u32);
}

pub static mut OS: Mutex<RefCell<Option<*mut dyn Os>>> = Mutex::new(RefCell::new(None));
