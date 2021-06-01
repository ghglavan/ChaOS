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
        let mut task = Task {stack_size, stack_addr, fn_addr, privileged, fp};
        
        unsafe {
            let task_stack_addr = stack_addr as *mut u32;
            let xpsr = task_stack_addr.offset((task.stack_size - 1) as isize);
            let pc = task_stack_addr.offset((task.stack_size - 2) as isize);
            let control = task_stack_addr.offset((task.stack_size - 17) as isize);
            let exc_return = task_stack_addr.offset((task.stack_size - 18) as isize);

            let control_val = task.get_ctrl();

            *xpsr = 0x0100_0000;
            *pc = task.fn_addr;
            *control = control_val;
            *exc_return = 0xFFFFFFFD;

            task.stack_addr = task_stack_addr.offset((task.stack_size - 18) as isize) as u32;
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
