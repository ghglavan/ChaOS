
use cortex_m::interrupt;
use cortex_m::peripheral::{syst::SystClkSource, SYST};

pub fn reset_timer(syst: &mut SYST, quanta_us: u32, ticks_for_10ms: u32) {
    syst.set_clock_source(SystClkSource::Core);
    let reload = (quanta_us / 1_000) * (ticks_for_10ms / 10);

    syst.set_reload(reload - 1);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();
}