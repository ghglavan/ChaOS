use crate::task;

#[inline(always)]
pub fn do_context_switch(prev_task: &mut task::Task, next_task: &task::Task, lr: u32) {
    unsafe {
        asm!(
            "mrs       r0, PSP",
            "tst       {0}, #0x10",
            "it        eq",
            "vstmdbeq  r0!, {{s16-s31}}",
            "mov       r2, {0}",
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
            in(reg) lr,
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
    let lr: u32 ;
    unsafe { asm!("mov {}, lr", out(reg) lr) };
    lr
}