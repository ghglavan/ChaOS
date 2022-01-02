use crate::task;

pub trait Scheduler<const N: usize> {
    fn init_with_tasks(tasks: [task::Task; N], quanta_us: u32) -> Self;
    fn get_initial_task_regs(&mut self) -> Option<(*const u32, u32, u32)>;
    fn get_switch_pair(&mut self) -> Option<(&mut task::Task, &mut task::Task)>;
    fn get_quanta_us(&self) -> u32;
}

pub struct RRScheduler<const N: usize> {
    tasks: [task::Task; N],
    current_task_idx: usize,
    quanta_us: u32,
}

impl<const N: usize> RRScheduler<N> {
    fn get_next_enabled_task_index(&self) -> Option<usize> {
        let old_idx = self.current_task_idx;
        let mut next_idx = self.current_task_idx;

        loop {
            if self.tasks[next_idx].state == task::TaskState::Enabled {
                return Some(next_idx);
            }

            next_idx = (next_idx + 1) % self.tasks.len();

            if next_idx == old_idx {
                return None;
            }
        }
    }
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

    fn get_initial_task_regs(&mut self) -> Option<(*const u32, u32, u32)> {
        let next_idx = self.get_next_enabled_task_index();
        if next_idx.is_none() {
            return None;
        }
        self.current_task_idx = next_idx.unwrap();

        let task = &mut self.tasks[self.current_task_idx];
        let psp = unsafe { (task.stack_addr as *const u32).offset(10) };
        let ctrl = task.get_ctrl();
        let exc_return = 0xFFFFFFFD;

        task.state = task::TaskState::Running;

        Some((psp, ctrl, exc_return))
    }

    fn get_switch_pair(&mut self) -> Option<(&mut task::Task, &mut task::Task)> {
        let next_idx = self.get_next_enabled_task_index();

        if next_idx.is_none() {
            return None;
        }

        let old_idx = self.current_task_idx;
        self.current_task_idx = next_idx.unwrap();

        if old_idx > self.current_task_idx {
            let (left, right) = self.tasks.split_at_mut(old_idx);
            Some((&mut right[0], &mut left[self.current_task_idx]))
        } else if self.current_task_idx > old_idx {
            let (left, right) = self.tasks.split_at_mut(self.current_task_idx);
            Some((&mut left[old_idx], &mut right[0]))
        } else {
            None
        }
    }

    fn get_quanta_us(&self) -> u32 {
        self.quanta_us
    }
}
