use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

use crate::task;

pub trait Os {
    fn reset_timer(&mut self);
    fn get_initial_task_regs(&mut self) -> Option<(*const u32, u32, u32)>;
    fn get_switch_pair(&mut self) -> Option<(&mut task::Task, &mut task::Task)>;
}

pub static mut OS: Mutex<RefCell<Option<*mut dyn Os>>> = Mutex::new(RefCell::new(None));
