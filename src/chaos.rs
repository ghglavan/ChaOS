use cortex_m::peripheral::{Peripherals, SYST};

use crate::os::Os;
use crate::task;
use crate::systick;
use crate::scheduler::Scheduler;

pub struct ChaOS< S: Scheduler<N>, const N: usize> {
    scheduler: S,
    ahb_freq: u32,
    ticks_for_10ms: u32,
    perifs: Peripherals
}

impl< S: Scheduler<N>, const N: usize,> Os for ChaOS<S, N> {
    fn reset_timer(&mut self) {
        systick::reset_timer(&mut self.perifs.SYST, self.scheduler.get_quanta_us(), self.ticks_for_10ms);
    }

    fn get_switch_pair(&mut self) -> (*mut u32, *const u32) {
        self.scheduler.get_switch_pair()
    }

    fn get_initial_task_regs(&self) -> (*const u32, u32, u32) {
        self.scheduler.get_initial_task_regs()
    }
}

impl<S: Scheduler<N>, const N: usize> ChaOS<S, N> {
    pub fn init(scheduler: S, ahb_freq: u32) -> Self {
        let ticks_for_10ms = match SYST::get_ticks_per_10ms() {
            0 =>  (ahb_freq as u32) / 10_000,
            x => x,
        };

        ChaOS {
            scheduler,
            ahb_freq,
            ticks_for_10ms,
            perifs: Peripherals::take().unwrap()
        }
    }
}