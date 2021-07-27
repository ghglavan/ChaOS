use cortex_m::interrupt;
use cortex_m_rt::exception;

#[exception]
fn SysTick() {
    cortex_m::peripheral::SCB::set_pendsv();
}

#[inline(always)]
pub fn do_pendsv_exception() {
    let lr = crate::asm::get_lr();
    let mut pair = None;
    interrupt::free(|cs| {
        unsafe {
            let (prev_task, next_task) = (*crate::os::OS.borrow(cs).borrow_mut().unwrap()).get_switch_pair();
            pair = Some((prev_task, next_task));
        }
    });    
    let (prev_task, next_task) = pair.unwrap();
    crate::asm::do_context_switch(prev_task, next_task, lr);
}

#[export_name = "PendSV"]
#[naked]
#[allow(non_snake_case)]
pub unsafe extern "C" fn PendSV() {
    do_pendsv_exception();
}

#[export_name = "SVCall"]
#[allow(non_snake_case)] 
pub unsafe extern "C" fn SVCall() {
    asm!(
        "tst       lr, #4",
        "ite       eq",
        "mrseq     r0, msp",
        "mrsne     r0, psp",
        "bl         {}",
        sym crate::syscalls::sv_call_handler,
    );
}