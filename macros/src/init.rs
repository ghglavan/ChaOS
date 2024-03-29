use crate::task;
use proc_macro2::TokenStream;
use quote::quote;

pub fn init(
    tasks: &[task::Task],
    scheduler: syn::Path,
    quanta_us: syn::LitInt,
    ahb_freq: syn::LitInt,
) -> TokenStream {
    let n_tasks = syn::LitInt::new(&tasks.len().to_string(), proc_macro2::Span::call_site());

    let tasks_init = tasks.iter().map(|t| {
        let stack_size = t.stack_size();
        let stack_addr = t.stack_name();
        let fn_addr = t.name.clone().unwrap();
        let privileged = t.privileged();
        let fp = t.fp();
        let id = t.id();
        let prio = t.prio();
        let disabled = t.disabled();

        quote! {
            ::chaos::task::Task::new(
                #stack_size as u32,
                #stack_addr.as_ptr() as u32,
                #id as u32,
                #prio as u32,
                #fn_addr as u32,
                #privileged,
                #fp,
                #disabled
            )
        }
    });

    let stacks = tasks.iter().map(|t| {
        let stack_name = t.stack_name();
        let stack_size = t.stack_size();
        quote! {
            static mut #stack_name: [u32; #stack_size] = [0; #stack_size];
        }
    });

    quote! {
        #[entry]
        fn main() -> ! {
            #(#stacks)*

            // this static is  going to be moved outside of main by the #[entry]
            // so  __CHAOS_OS_OBJ becomes a &mut Option<_>
            static mut __CHAOS_OS_OBJ: Option<::chaos::chaos::ChaOS::<#scheduler<#n_tasks>, #n_tasks>> = None;

            use cortex_m::interrupt::{self, Mutex};

            let tasks = unsafe { [#(#tasks_init),*] };
            let sched: #scheduler<#n_tasks> = #scheduler::<#n_tasks>::init_with_tasks(tasks, #quanta_us);

            unsafe {
                *__CHAOS_OS_OBJ = Some(::chaos::chaos::ChaOS::<#scheduler<#n_tasks>, #n_tasks>::init(sched, #ahb_freq));
            }

            interrupt::free(|cs| {
                unsafe {
                    chaos::os::OS
                        .borrow(cs)
                        .replace(Some(__CHAOS_OS_OBJ.as_mut().unwrap() as *mut dyn ::chaos::os::Os));
                }
            });

            unsafe {
                interrupt::enable();
                asm!("svc #0");
            }

            loop {
                continue;
            }
        }
    }
}
