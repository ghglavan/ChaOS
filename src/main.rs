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

static mut TCBS: [TaskControlBlock; TASKS] = [TaskControlBlock{stack_ptr: 0 as *mut u32}; TASKS];
static mut CURRENT_TCB_INDEX: usize = 2;

static mut TCB_STACK0: [u32; TCB_STACK_SIZE] = [0; TCB_STACK_SIZE];
static mut TCB_STACK1: [u32; TCB_STACK_SIZE] = [0; TCB_STACK_SIZE];
static mut TCB_STACK2: [u32; TCB_STACK_SIZE] = [0; TCB_STACK_SIZE];

fn do_switch() {
     unsafe {
        let current_tcb = &mut TCBS[CURRENT_TCB_INDEX];
        CURRENT_TCB_INDEX = (CURRENT_TCB_INDEX + 1) % TASKS;
        let next_tcb = &mut TCBS[CURRENT_TCB_INDEX];

        asm!(
             "mrs {0}, psp", // load psp to r0
             "stmdb {0}!, {{r4-r11}}", // push r4-r11 adn set r0 to then new stack
             "ldmia {1}!, {{r4-r11}}",
             "msr psp, {1}",
             inout(reg) current_tcb.stack_ptr, inout(reg) next_tcb.stack_ptr
        );
    }
}

#[exception]
fn SysTick() {
    interrupt::disable();
    do_switch();
    unsafe {interrupt::enable();}
    
    unsafe {
        asm!("ldr lr, ={LD}", // load lr with 0xfffffff9 in order to return from psp 
             "pop {{r0-r3}}", // pop the top 4 values so we dont fill the main stack
             "bx lr", // branch to lr so we finnish the context switch
             LD = const 0xfffffffdu32
             ); 
    }
}

fn task0() {
    
    loop {
        unsafe {TASK0_COUNTER += 1;} 
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
        
        TCBS[0].stack_ptr = (&mut TCB_STACK0).as_mut_ptr().offset((TCB_STACK_SIZE - 16) as isize); 
        TCBS[1].stack_ptr = (&mut TCB_STACK1).as_mut_ptr().offset((TCB_STACK_SIZE - 16) as isize); 
        TCBS[2].stack_ptr = (&mut TCB_STACK2).as_mut_ptr().offset((TCB_STACK_SIZE - 8) as isize); 

        TCB_STACK0[TCB_STACK_SIZE - 1] = 0x0100_0000;
        TCB_STACK1[TCB_STACK_SIZE - 1] = 0x0100_0000;
        TCB_STACK2[TCB_STACK_SIZE - 1] = 0x0100_0000;

        TCB_STACK0[TCB_STACK_SIZE - 2] = task0 as u32;
        TCB_STACK1[TCB_STACK_SIZE - 2] = task1 as u32;
        TCB_STACK2[TCB_STACK_SIZE - 2] = task2 as u32;
        
        CURRENT_TCB_INDEX = 2;
       
        let current_tcb = &TCBS[CURRENT_TCB_INDEX];
        // set psp to task2 so we can store psp in the context switch
        asm!("msr psp, {}", in(reg) current_tcb.stack_ptr);
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

}


#[entry]
fn main() -> ! {

    setup();

    loop {
        continue;
    }
}
