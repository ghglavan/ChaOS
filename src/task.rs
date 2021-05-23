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
    fn new(stack_size: u32, stack_addr: u32, fn_addr: u32, privileged: bool, fp: bool) -> Self {
        return Task {stack_size, stack_addr, fn_addr, privileged, fp}
    }
}
