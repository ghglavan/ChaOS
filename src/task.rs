#[derive(Copy, Clone)]
#[repr(C)]
pub struct Task {
    pub stack_size: u32,
    pub stack_addr: u32,
    pub fn_addr: u32,
    pub privileged: bool,
    pub fp: bool,
}

impl Task {
    pub fn new(stack_size: u32, stack_addr: u32, fn_addr: u32, privileged: bool, fp: bool) -> Self {
        let mut task = Task {
            stack_size,
            stack_addr,
            fn_addr,
            privileged,
            fp,
        };

        // align stack_address to 8
        let stack_addr = ((stack_addr + stack_size) / 8) * 8;

        unsafe {
            let task_stack_addr = stack_addr as *mut u32;
            let xpsr = task_stack_addr.offset(-1);
            let pc = task_stack_addr.offset(-2);
            let control = task_stack_addr.offset(-17);
            let exc_return = task_stack_addr.offset(-18);

            let control_val = task.get_ctrl();

            *xpsr = 0x0100_0000;
            *pc = task.fn_addr;
            *control = control_val;
            *exc_return = 0xFFFFFFFD;

            task.stack_addr = task_stack_addr.offset(-18) as u32;
        }

        task
    }

    pub fn get_ctrl(&self) -> u32 {
        if self.privileged {
            0x2
        } else {
            0x3
        }
    }
}
