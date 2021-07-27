use crate::task;

pub trait Scheduler<const N: usize> {
    fn init_with_tasks(tasks: [task::Task; N], quanta_us: u32) -> Self;
    fn get_initial_task_regs(&self) -> (*const u32, u32, u32);
    fn get_switch_pair(&mut self) -> (&mut task::Task, &task::Task);
    fn get_quanta_us(&self) -> u32;
}

pub struct RRScheduler<const N: usize> {
    tasks: [task::Task; N],
    current_task_idx: usize,
    quanta_us: u32,
}

impl<const N: usize> Scheduler<N> for RRScheduler<N> {
    fn init_with_tasks(tasks: [task::Task; N], quanta_us: u32) -> Self {
        let current_task_idx = 0;

        RRScheduler {
            tasks,
            current_task_idx,
            quanta_us,
        }
    }

    fn get_initial_task_regs(&self) -> (*const u32, u32, u32) {
        let task = &self.tasks[self.current_task_idx];
        let psp = unsafe { (task.stack_addr as *const u32).offset(10) };
        let ctrl = task.get_ctrl();
        let exc_return = 0xFFFFFFFD;

        (psp, ctrl, exc_return)
    }

    fn get_switch_pair(&mut self) -> (&mut task::Task, &task::Task) {
        self.current_task_idx = (self.current_task_idx + 1) % self.tasks.len();
        if self.current_task_idx == 0 {
            let (left, right) = self.tasks.split_at_mut(self.current_task_idx + 1);
            (&mut right[right.len() - 1], &left[0])
        } else {
            let (left, right) = self.tasks.split_at_mut(self.current_task_idx);
            (&mut left[left.len() - 1], &right[0])
        }
    }

    fn get_quanta_us(&self) -> u32 {
        self.quanta_us
    }
}
