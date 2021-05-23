use proc_macro2::TokenStream;
use quote::quote;

pub fn exceptions() -> TokenStream {
    quote! {
        #[exception]
        fn SysTick() {
            cortex_m::peripheral::SCB::set_pendsv();
        }

        #[export_name = "PendSV"]
        pub unsafe extern "C" fn PendSV() {
            interrupt::free(|cs| {
                unsafe {
                    let (prev_stack, next_stack) = (*::chaos::os::OS.borrow(cs).borrow_mut().unwrap()).get_switch_pair();
                    ::chaos::asm::do_context_switch(prev_stack, next_stack);
                }
            });
        }

        #[export_name = "SVCall"]
        pub unsafe extern "C" fn SVCall() {
            asm!(
                "tst       lr, #4",
                "ite       eq",
                "mrseq     r0, msp",
                "mrsne     r0, psp",
                "bl         {}",
                sym ::chaos::syscalls::sv_call_handler,
            );
        }
    }
}