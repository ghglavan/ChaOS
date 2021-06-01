use proc_macro::TokenStream;

use syn::spanned::Spanned;
use quote::quote;

mod mod_args;
mod task;
mod init;

#[proc_macro_attribute]
pub fn os(args: TokenStream, input: TokenStream) -> TokenStream {
    let rest = syn::parse_macro_input!(input as syn::ItemMod);
    let args = syn::parse_macro_input!(args as mod_args::ModArgs);

    let mod_name = rest.ident.clone();
    let mut tasks = Vec::new();
    let mut init_function = None;

    let content = if let Some((_, mut content)) = rest.content {
        for item in content.iter_mut() {
            match item {
                syn::Item::Fn(f) => {
                    let mut new_attrs = Vec::new();
                    for attr in f.attrs.iter() {
                        if attr
                            .path
                            .is_ident(&syn::Ident::new("task", proc_macro2::Span::call_site()))
                        {
                            let mut task = match attr.parse_args::<task::Task>() {
                                Ok(t) => {
                                    t
                                }
                                Err(e) => return e.into_compile_error().into(),
                            };
                            task.name = Some(f.sig.ident.clone());
                            tasks.push(task);
                        } else if attr.path.is_ident(&syn::Ident::new("init", proc_macro2::Span::call_site())) {
                            if init_function.is_some() {
                                return syn::Error::new(f.span(), "only one init function should be defined").into_compile_error().into();
                            }

                            match f.sig.output {
                                syn::ReturnType::Default => init_function = Some(f.sig.ident.clone()),
                                _ => return syn::Error::new(f.span(), "init function should have the default return AND SHOULD RETURN!!").into_compile_error().into()
                            }
                        } else {
                            new_attrs.push(attr.clone());
                        }
                    }
                    f.attrs = new_attrs;
                }
                _ => {}
            }
        }

        content
    } else {
        Vec::new()
    };

    let stacks = tasks.iter().map(|t| {
        let stack_name = t.stack_name();
        let stack_size = t.stack_size();
        quote! {
            static mut #stack_name: [u32; #stack_size] = [0; #stack_size];
        }
    });

    let n_tasks = syn::LitInt::new(&tasks.len().to_string(), proc_macro2::Span::call_site());
    let content = content.iter();
    let quanta_us = syn::LitInt::new(&args.quanta_us.to_string(), proc_macro2::Span::call_site());
    let ahb_freq = syn::LitInt::new(&args.ahb_freq.to_string(), proc_macro2::Span::call_site());
    let init = init::init(&tasks, args.scheduler.clone(), quanta_us, ahb_freq);

    (quote! {
        use ::chaos::scheduler::Scheduler;
        use ::chaos::os::Os;

        #(#stacks)*

        #(#content)*

        #init
    })
    .into()
}
