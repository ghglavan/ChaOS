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
    next_tcb_ptr: *mut TaskControlBlock,
}

static mut TASK1_COUNTER: u32 = 0;
static mut TASK2_COUNTER: u32 = 0;
static mut TASK3_COUNTER: u32 = 0;

static mut TCBS: [TaskControlBlock; TASKS] = [TaskControlBlock{stack_ptr: 0 as *mut u32, next_tcb_ptr: 0 as *mut TaskControlBlock}; TASKS];
static mut CURRENT_TCB: *mut TaskControlBlock = 0 as *mut TaskControlBlock; 
static mut TCB_STACK1: [u32; TCB_STACK_SIZE] = [0; TCB_STACK_SIZE];
static mut TCB_STACK2: [u32; TCB_STACK_SIZE] = [0; TCB_STACK_SIZE];
static mut TCB_STACK3: [u32; TCB_STACK_SIZE] = [0; TCB_STACK_SIZE];

#[exception]
unsafe fn SVCall() {
    
    asm!("cpsid i"); // disable interrupts
 
    asm!("ldr r0, ={}", const 0xfffffffdu32);
    asm!("push {{r0}}");
    asm!("mrs r0, msp");
    asm!("push {{r0}}");
   
        // we could use the interrupt::disable and interrupt::enable()
    // but this will mess up with the lr
    asm!("ldr r0, ={}", sym CURRENT_TCB); // load into r0 the addr of current_tcb
    asm!("ldr r2, [r0]"); // load into r2 current_tcb (the stack pointer)
    asm!("ldr r3, [r2]");
    // switch to using psp
    asm!("mrs r0, control"); // read control to r0
    asm!("orrs r0, r0, #2"); // set SPSEL
    asm!("msr control, r0"); // write back to control
    asm!("isb"); // sync stuff
    
    asm!("msr psp, r3"); // load the current_tcp->stack_ptr to psp
    //asm!("pop {{r4-r11}}"); // pop r4-r11 from the stack
    //asm!("pop {{r0-r3}}"); // pop r0-r3 from the stack
    //asm!("pop {{r12}}"); // pop r12 from the stack
    //asm!("add sp, sp, #4"); // skip r13
    //asm!("pop {{lr}}"); // pop lr from the stack
    //asm!("add sp, sp, #4");
    asm!("cpsie i"); // enable interrupt
}

#[exception]
fn SysTick() {
    hprintln!("hello");

    unsafe {
        // we could use the interrupt::disable and interrupt::enable()
        // but this will mess up with the lr
        asm!("cpsid i", // disable interrupts
             "mrs r0, psp", // load psp to r0
             "stmdb r0!, {{r4-r11}}", // push r4-r11 adn set r0 to then new stack
             "ldr r4, ={TCB}", // set r4 to point to current_tcb
             "ldr r5, [r4]", // set r5 to the first value of current_tcb (stack_ptr)
             "str r0, [r5]", // store the new stack ptr to current_tcb->stack_ptr
             "ldr r5, [r5, #4]", // set r5 to the second value of current_tcb (next_tcb_ptr)
             "str r5, [r4]", // store current_tcb.next_tcb_ptr to r4 (current_tcp)
             "ldr r0, [r5]", // set r0 to the new current_tcb.stack_ptr
             "ldmia r0!, {{r4-r11}}", // pop r4-r11 from the current_tcb.stack_ptr
             "msr psp, r0", // set psp to the stack of the new process (r0)
             "cpsie i", // enable interrupt
             "ldr lr, ={LD}", // load lr with 0xfffffff9 in order to return from psp 
             "pop {{r0-r1}}", // pop the top 4 values so we dont fill the main stack
             "bx lr", // branch to lr so we finnish the context switch
             TCB = sym CURRENT_TCB, LD = const 0xfffffffdu32); 
    }
}

fn task0() {
    
    loop {
        unsafe {TASK1_COUNTER += 1;} 
        continue;
    }
}

fn task1() {

    loop {
        unsafe {TASK2_COUNTER += 1;}
        continue;
    }
}

fn task2() {

    loop {
        unsafe {TASK3_COUNTER += 1;}        
        continue;
    }
}

fn setup() {
    interrupt::disable();

    unsafe {
        /*for i in 0..TASKS {
            if i == TASKS - 1 {
                (*TCBS[i]).next_tcb_ptr = TCBS[0];    
            } else {
                (*TCBS[i]).next_tcb_ptr = TCBS[i + 1];    
            }
            (*TCBS[i]).stack_ptr = &mut TCB_STACKS[i][TCB_STACK_SIZE - 16] as *mut u32;
            TCB_STACKS[i][TCB_STACK_SIZE - 1] = 0x0100_0000;
        }*/
        TCBS[0].next_tcb_ptr = &mut TCBS[1] as *mut TaskControlBlock;
        TCBS[0].stack_ptr = &mut TCB_STACK1[TCB_STACK_SIZE - 16] as *mut u32;
        TCB_STACK1[TCB_STACK_SIZE - 1] = 0x0100_0000;

        TCBS[1].next_tcb_ptr = &mut TCBS[2] as *mut TaskControlBlock;
        TCBS[1].stack_ptr = &mut TCB_STACK2[TCB_STACK_SIZE - 16] as *mut u32;
        TCB_STACK2[TCB_STACK_SIZE - 1] = 0x0100_0000;

        TCBS[2].next_tcb_ptr = &mut TCBS[0] as *mut TaskControlBlock;
        TCBS[2].stack_ptr = &mut TCB_STACK3[TCB_STACK_SIZE - 8] as *mut u32;
        TCB_STACK3[TCB_STACK_SIZE - 1] = 0x0100_0000;

        TCB_STACK1[TCB_STACK_SIZE - 2] = task0 as u32;
        TCB_STACK2[TCB_STACK_SIZE - 2] = task1 as u32;
        TCB_STACK3[TCB_STACK_SIZE - 2] = task2 as u32;
        CURRENT_TCB = &mut TCBS[2] as *mut TaskControlBlock;
        asm!("ldr r0, ={}", sym CURRENT_TCB); // load into r0 the addr of current_tcb
        asm!("ldr r2, [r0]"); // load into r2 current_tcb (the stack pointer)
        asm!("ldr r3, [r2]");
        asm!("msr psp, r3");
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


    //unsafe {
    //    asm!("svc #0");
    //}
    
}


#[entry]
fn main() -> ! {

    setup();

    loop {
        continue;
        // your code goes here
    }
}
