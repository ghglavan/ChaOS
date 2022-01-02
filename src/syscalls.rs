use cortex_m::interrupt;
use cortex_m::register;

use crate::asm;
use crate::os::OS;

#[derive(Copy, Clone)]
enum Syscalls {
    Setup = 0,
    Sleep = 1,
    Unknown,
}

impl Syscalls {
    fn val(&self) -> u8 {
        *self as u8
    }
}

impl From<u8> for Syscalls {
    fn from(val: u8) -> Self {
        match val {
            0 => Syscalls::Setup,
            1 => Syscalls::Sleep,
            _ => Syscalls::Unknown,
        }
    }
}

fn handle_setup_call() {
    let mut task_regs = (0 as *const u32, 0, 0);
    interrupt::free(|cs| unsafe {
        (*OS.borrow(cs).borrow_mut().unwrap()).reset_timer();
        task_regs = (*OS.borrow(cs).borrow_mut().unwrap()).get_initial_task_regs();
    });
    let (psp, ctrl, exc_return) = task_regs;
    asm::do_setup(psp, ctrl, exc_return);
}

fn handle_sleep_call() {
    interrupt::free(|cs| unsafe {
        cortex_m::peripheral::SCB::set_pendsv();
        (*OS.borrow(cs).borrow_mut().unwrap()).reset_timer();
    });
}

#[export_name = "sv_call_handler"]
#[inline(always)]
pub unsafe extern "C" fn sv_call_handler(stack: *mut u32) {
    // we use 8 if we are working with msp since the call to the SVC handler
    // will push 2 registers to the stack ...
    // to fix this we need to be able to define a function inside asm!
    let offset = if stack == register::psp::read() as *mut u32 {
        6
    } else {
        8
    };

    let stack_pc = stack.offset(offset);
    let code = (*stack_pc) as *mut u8;
    let offset = code.offset(-2);
    let syscall_code: Syscalls = (*offset).into();

    match syscall_code {
        Syscalls::Setup => {
            handle_setup_call();
        }
        Syscalls::Sleep => {
            handle_sleep_call();
        }
        Syscalls::Unknown => {}
    }
}

#[inline(always)]
pub fn sleep() {
    unsafe {
        asm!("svc #1");
    }
}
