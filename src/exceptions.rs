use cortex_m::interrupt;
use cortex_m_rt::exception;

use crate::task;

#[exception]
fn SysTick() {
    cortex_m::peripheral::SCB::set_pendsv();
}

#[inline(always)]
pub fn do_pendsv_exception() {
    let lr = crate::asm::get_lr();

    let pair = interrupt::free(|cs| unsafe {
        (*crate::os::OS.borrow(cs).borrow_mut().unwrap()).get_switch_pair()
    });

    if pair.is_none() {
        return;
    }

    let (prev_task, next_task) = pair.unwrap();

    if prev_task.state == crate::task::TaskState::Running {
        prev_task.state = crate::task::TaskState::Enabled;
    }
    next_task.state = crate::task::TaskState::Running;

    crate::asm::do_context_switch(prev_task, next_task, lr);
}

#[export_name = "PendSV"]
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
        "bl         sv_call_handler",
    );
}
