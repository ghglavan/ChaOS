use cortex_m::interrupt;
use cortex_m_rt::exception;

#[exception]
fn SysTick() {
    cortex_m::peripheral::SCB::set_pendsv();
}

#[export_name = "PendSV"]
#[allow(non_snake_case)]
pub unsafe extern "C" fn PendSV() {
    let lr = crate::asm::get_lr();
    interrupt::free(|cs| {
        unsafe {
            let (prev_task, next_task) = (*crate::os::OS.borrow(cs).borrow_mut().unwrap()).get_switch_pair();
            crate::asm::do_context_switch(prev_task, next_task, lr);
        }
    });
}

#[allow(non_snake_case)] 
#[export_name = "SVCall"]
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