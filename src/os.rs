use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

pub trait Os {
    fn reset_timer(&mut self);
    fn get_switch_pair(&mut self) -> (*mut u32, *const u32);
}

pub static mut OS: Mutex<RefCell<Option<*mut dyn Os>>> = Mutex::new(RefCell::new(None));
