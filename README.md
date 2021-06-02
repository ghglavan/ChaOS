# ChaOS

Preemptive Real Time OS

Disclaimer: The purpose of this project is learning. It should not be used for anything serious. If you are looking for
an environment for running some production cortex-m code, you should check [cortex-m-rtic](https://github.com/rtic-rs/cortex-m-rtic) or [TockOS](https://www.tockos.org/)

# How it works

For an example you should check [chaos-example](https://github.com/ghglavan/chaos-test)
Heavily inpired by [cortex-m-rtic](https://github.com/rtic-rs/cortex-m-rtic), ChaOS uses procedural macros for defining the context and tasks:

```rust
#[chaos::os(quanta_us = 10_000, scheduler = chaos::scheduler::RRScheduler)]
mod chaos {
    #[task(stack_size = 500)]
    fn task0() {
    }
}
```
All os related attributes are specified in the os macro. Available attributes:
* `quanta_us`: The interval for a SysTick in microseconds (default to 10_000)
* `schdeuler`: The cheduler to be used (mandatory)
* `ahb_freq`: The frequency of the bus that drives the SysTick (default to `16_000_000` as I only tested for `STM32f4DISCOVERY`). This is only used if `SYST::get_ticks_per_10ms` returns 0

Inside the mod you can define the tasks that will be scheduled. Available attributes:
* `stack_size`: The size of the stack for the task (mandatory)
* `privileged`: Flag set the task should run in privileged mode (default disabled)
* `fp`: Flag set if the task should start with floating point context enabled (default disabled)

ChaOS is preemptive, it will construct an array from the provided tasks and will pass it to the scheduler. At every SysTick
interrupt or sleep system call, the scheduler will decide which task to schedule next and the os will do the context switch
very similar to the one presented in [The Definitive Guide to ARM Cortex -M3 and Cortex-M4 Processors](#References)

Everything is static right now. You can not add task dinamically or start/stop existing ones.

# Syscalls

Available syscalls:
* Sleep. The name is not really good. What this does is that it preempts the task.

# Future plans

- [ ] starting/stopping tasks
- [ ] dynamic memory
- [ ] dynamic tasks
- [ ] MMU
- [ ] os objects (mutexes, semaphores, condition variables, etc.)
- [ ] drivers 

# References
[1]: The Definitive Guide to ARM Cortex -M3 and Cortex-M4 Processors by Joseph Yiu