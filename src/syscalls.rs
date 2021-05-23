use cortex_m::interrupt;
use cortex_m::register;

use crate::os::OS;
use crate::asm;

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
    interrupt::free(|cs| {
        unsafe {
            (*OS.borrow(cs).borrow_mut().unwrap()).reset_timer();

            let (psp, ctrl, exc_return) = (*OS.borrow(cs).borrow_mut().unwrap()).get_initial_task_regs();
            asm::do_setup(psp, ctrl, exc_return);
        }
    });

}

fn handle_sleep_call() {
    interrupt::free(|cs| {
        unsafe {
            let (prev_stack, next_stack) = (*OS.borrow(cs).borrow_mut().unwrap()).get_switch_pair();
            asm::do_context_switch(prev_stack, next_stack);
            (*OS.borrow(cs).borrow_mut().unwrap()).reset_timer();
        }
    });
}

pub fn sv_call_handler(stack: *mut u32) {
    unsafe {
        let stack_pc = stack.offset(6);
        let code = (*stack_pc) as *mut u8;
        let offset = code.offset(-2);
        let syscall_code: Syscalls = (*offset).into();

        let privileged = if stack == register::psp::read() as *mut u32 {
            false
        } else {
            true
        };

        match syscall_code {
            Syscalls::Setup => {
                if !privileged {
                    return;
                }

                handle_setup_call();
            }
            Syscalls::Sleep => {
                handle_sleep_call();
            }
            Syscalls::Unknown => {}
        }
    }
}