use crate::task;

pub trait Scheduler<const N: usize> {
    fn init_with_tasks(tasks: [task::Task; N], quanta_us: u32) -> Self;
    fn get_switch_pair(&mut self) -> (*mut u32, *const u32);
    fn get_quanta_us(&self) -> u32;
}

pub struct RRScheduler<const N: usize> {
    tasks: [task::Task; N],
    current_task_idx: usize,
    quanta_us: u32
}

impl<const N: usize> Scheduler<N> for RRScheduler<N> {
    fn init_with_tasks(mut tasks: [task::Task; N], quanta_us: u32) -> Self {
        for task in tasks.iter_mut() {
            unsafe {
                let task_stack_addr = task.stack_addr as *mut u32;
                let xpsr = task_stack_addr.offset((task.stack_size - 1) as isize);
                let pc = task_stack_addr.offset((task.stack_size - 2) as isize);
                let control = task_stack_addr.offset((task.stack_size - 17) as isize);
                let exc_return = task_stack_addr.offset((task.stack_size - 18) as isize);

                let control_val = if task.privileged {
                    0x2
                } else {
                    0x3
                };

                *xpsr = 0x0100_0000;
                *pc = task.fn_addr;
                *control = control_val;
                *exc_return = 0xFFFFFFFD;

                task.stack_addr = task_stack_addr.offset((task.stack_size - 18) as isize) as u32;
            }
        }

        let current_task_idx = 0;

        RRScheduler {tasks, current_task_idx, quanta_us}
    }

    fn get_switch_pair(&mut self) -> (*mut u32, *const u32) {
        unsafe {
            let mut current_stack = self.tasks[self.current_task_idx].stack_addr as *mut u32;
            self.current_task_idx = (self.current_task_idx + 1 ) % self.tasks.len();
            let next_stack = self.tasks[self.current_task_idx].stack_addr as *mut u32;

            (current_stack, next_stack)
        }
    }

    fn get_quanta_us(&self) -> u32 {
        self.quanta_us
    }
}