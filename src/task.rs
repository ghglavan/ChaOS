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
    pub fn get_ctrl(&self) -> u32 {
        if self.privileged {
            0x2
        } else {
            0x3
        }
    }
}
