use crate::task;

// pub mod exceptions {
//     use cortex_m::interrupt;

//     #[no_mangle]
//     #[inline(always)]
//     pub unsafe extern "C" fn do_pendsv_exception() {
//         let lr = crate::asm::get_lr();
//         let mut pair = None;
//         interrupt::free(|cs| {
//             unsafe {
//                 let (prev_task, next_task) = (*crate::os::OS.borrow(cs).borrow_mut().unwrap()).get_switch_pair();
//                 pair = Some((prev_task, next_task));
//             }
//         });
//         let (prev_task, next_task) = pair.unwrap();
//         crate::asm::do_context_switch(prev_task, next_task, lr);
//     }

//     global_asm!(r#"
//     .section txt
//     .global PendSV
//     foo:
//         bl do_pendsv_exception
//     "#);
// }

// extern "C" {
//     fn PendSV();
// }

#[inline(always)]
pub fn do_context_switch(prev_task: &mut task::Task, next_task: &task::Task, lr: u32) {
    unsafe {
        asm!(
            "push      {{r3}}",
            "mrs       r0, PSP",
            "tst       r2, #0x10",
            "it        eq",
            "vstmdbeq  r0!, {{s16-s31}}",
            "mrs       r3, control",
            "stmdb     r0!, {{r2-r11}}",
            "ldmia     r1!, {{r2-r11}}",
            "tst       lr, #0x10",
            "it        eq",
            "vldmiaeq  r1!, {{s16-s31}}",
            "mov       lr, r2",
            "msr       control, r3",
            "isb",
            "pop       {{r3}}",
            "msr       psp, r1",
            in("r2") lr,
            out("r0") prev_task.stack_addr,
            in("r1") next_task.stack_addr,
        );
    }
}

#[inline(always)]
pub fn do_setup(psp: *const u32, ctrl: u32, exc_return: u32) {
    unsafe {
        asm!(
            "msr psp, {0}",
            "msr control, {1}",
            "isb",
            "mov lr, {2}",
            "bx lr",
            in(reg) psp,
            in(reg) ctrl,
            in(reg) exc_return
        );
    }
}

#[inline(always)]
pub fn get_lr() -> u32 {
    let lr: u32;
    unsafe { asm!("mov {}, lr", out(reg) lr) };
    lr
}
