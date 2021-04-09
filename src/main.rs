#![no_std]
#![no_main]
#![feature(asm)]

pub use stm32f3_discovery::{leds::Leds, stm32f3xx_hal, switch_hal};
use stm32f3xx_hal::prelude::*;
pub use stm32f3xx_hal::{
    delay::Delay,
    gpio::{gpioe, Output, PushPull},
    hal::blocking::delay::DelayMs,
    stm32,
};

use core::mem::MaybeUninit;

use panic_semihosting as _; // you can put a breakpoint on `rust_begin_unwind` to catch panicuse cortex_m::asm;
use cortex_m_rt::{entry, exception};
use cortex_m_semihosting::*;
use cortex_m::interrupt;
use cortex_m::Peripherals;
use cortex_m::peripheral::{SYST, syst::SystClkSource};


const TASKS: usize = 3;
const TCB_STACK_SIZE: usize = 300;
const QUANTA_US: u32 = 100 * 1_000; // 10 ms 
const BUS_FREQ: u32 = 16 * 1_000_000;
const US_SCALER: u32 = 1_000_000;

#[repr(C)]
#[derive(Copy, Clone)]
struct TaskControlBlock {
    stack_ptr: *mut u32, 
}

static mut TASK0_COUNTER: u32 = 0;
static mut TASK1_COUNTER: u32 = 0;
static mut TASK2_COUNTER: u32 = 0;

static mut TCBS: [TaskControlBlock; TASKS] = [TaskControlBlock {stack_ptr: 0 as *mut u32}; TASKS];
static mut CURRENT_TCB_INDEX: usize = 2;
static mut NEXT_TCB_INDEX: usize = 0;

static mut TCB_STACK0: [u32; TCB_STACK_SIZE] = [0; TCB_STACK_SIZE];
static mut TCB_STACK1: [u32; TCB_STACK_SIZE] = [0; TCB_STACK_SIZE];
static mut TCB_STACK2: [u32; TCB_STACK_SIZE] = [0; TCB_STACK_SIZE];

#[exception]
fn SysTick() {
    cortex_m::peripheral::SCB::set_pendsv();
}


// wo dont use the classic exception here because we ned the lr in order to check the
// fp status of the task. Using the #[exception] macro will generate
//
// #[doc(hidden)]
// #[export_name = "PendSV"]
// pub unsafe extern "C" fn __cortex_m_rt_PendSV_trampoline() {
//     __cortex_m_rt_PendSV()
// }
// fn __cortex_m_rt_PendSV() {
//		... our function ...
// }
// 
// 
// this does not help us becase the trampoline is going to change the lr so the check
// 'tst  lr, #0x10' will give us a bogus value because it uses the trampoline lr
#[export_name = "PendSV"]
pub unsafe extern "C" fn PendSV() {
    unsafe {
        let prev_tcb = &mut TCBS[CURRENT_TCB_INDEX];
        CURRENT_TCB_INDEX = (CURRENT_TCB_INDEX + 1) % TASKS;
        let next_tcb = &mut TCBS[CURRENT_TCB_INDEX];
     
        asm!(
             "mrs       r0, PSP",
             "tst       lr, #0x10",
             "it        eq",
             "vstmdbeq  r0!, {{s16-s31}}",
             "mov       r2, lr",
             "mrs       r3, control",
             "stmdb     r0!, {{r2-r11}}",
             "ldmia     r1!, {{r2-r11}}",
             "mov       lr, r2",
             "msr       control, r3",
             "isb",
             "tst       lr, #0x10",
             "it        eq",
             "vldmiaeq  r1!, {{s16-s31}}",
             "msr       psp, r1",
             inout("r0") prev_tcb.stack_ptr, inout("r1") next_tcb.stack_ptr
            );
    }
}

fn sv_call_handler(stack: *mut u32) {
    
    let asdasd = unsafe {
        *(*stack.offset(6) as *mut u8).offset(-2)
    };

    loop {
        continue;
    } 
}

#[export_name = "SVCall"]
pub unsafe extern "C" fn SVCall() {
    unsafe {
        asm!(
             "tst       lr, #4",
             "ite       eq",
             "mrseq     r0, msp",
             "mrsne     r0, psp",
             "b         {}",
             sym sv_call_handler
             );
        }
}

fn task0() {
    
    loop {
        unsafe {
            TASK0_COUNTER += 1;
            asm!("svc #9");
        } 
        
        continue;
    }
}

fn task1() {

    loop {
        unsafe {TASK1_COUNTER += 1;}
        continue;
    }
}

fn task2() {

    loop {
        unsafe {TASK2_COUNTER += 1;}        
        continue;
    }
}

fn setup() {
    interrupt::disable();

    unsafe {
        
        TCBS[0].stack_ptr = (&mut TCB_STACK0).as_mut_ptr().offset((TCB_STACK_SIZE - 18) as isize); 
        TCBS[1].stack_ptr = (&mut TCB_STACK1).as_mut_ptr().offset((TCB_STACK_SIZE - 18) as isize); 
        TCBS[2].stack_ptr = (&mut TCB_STACK2).as_mut_ptr().offset((TCB_STACK_SIZE - 18) as isize); 

        TCB_STACK0[TCB_STACK_SIZE - 1] = 0x0100_0000;
        TCB_STACK1[TCB_STACK_SIZE - 1] = 0x0100_0000;
        TCB_STACK2[TCB_STACK_SIZE - 1] = 0x0100_0000;

        TCB_STACK0[TCB_STACK_SIZE - 2] = task0 as u32;
        TCB_STACK1[TCB_STACK_SIZE - 2] = task1 as u32;
        TCB_STACK2[TCB_STACK_SIZE - 2] = task2 as u32;
        
        TCB_STACK0[TCB_STACK_SIZE - 17] = 0x3; // initial CONTROL: unprivileged, PSP, no fp
        TCB_STACK1[TCB_STACK_SIZE - 17] = 0x3; // initial CONTROL: unprivileged, PSP, no fp
        TCB_STACK2[TCB_STACK_SIZE - 17] = 0x3; // initial CONTROL: unprivileged, PSP, no fp

        TCB_STACK0[TCB_STACK_SIZE - 18] = 0xFFFFFFFD;
        TCB_STACK1[TCB_STACK_SIZE - 18] = 0xFFFFFFFD;
        TCB_STACK2[TCB_STACK_SIZE - 18] = 0xFFFFFFFD;
        
        CURRENT_TCB_INDEX = 0;
       
	}


    unsafe {
        interrupt::enable();
    }

    // setup  systick
    let mut syst = Peripherals::take().unwrap().SYST;
    syst.set_clock_source(SystClkSource::Core);
    let reload = QUANTA_US * (BUS_FREQ / US_SCALER);
    syst.set_reload(reload - 1);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();
    
    unsafe {
        let current_tcb = &TCBS[CURRENT_TCB_INDEX];
         
        let ctrl = 0x3;
        asm!(
            "msr psp, {0}",
            "msr control, {1}",
            in(reg) current_tcb.stack_ptr,
            in(reg) ctrl
            );
    }
       
    task0();

}


#[entry]
fn main() -> ! {

    setup();

    loop {
        continue;
    }
}
