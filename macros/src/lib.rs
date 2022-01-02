use proc_macro::TokenStream;

use quote::quote;
use syn::spanned::Spanned;

mod init;
mod mod_args;
mod task;

enum TaskIdState {
    Inactive,
    Active,
}

#[proc_macro_attribute]
pub fn os(args: TokenStream, input: TokenStream) -> TokenStream {
    let rest = syn::parse_macro_input!(input as syn::ItemMod);
    let args = syn::parse_macro_input!(args as mod_args::ModArgs);

    let mut tasks = Vec::new();
    let mut init_function = None;

    let content = if let Some((_, mut content)) = rest.content {
        for item in content.iter_mut() {
            match item {
                syn::Item::Fn(f) => {
                    let mut new_attrs = Vec::new();
                    let mut ids = 0;
                    let mut task_ids = None;
                    for attr in f.attrs.iter() {
                        if attr
                            .path
                            .is_ident(&syn::Ident::new("task", proc_macro2::Span::call_site()))
                        {
                            let mut task = match attr.parse_args::<task::Task>() {
                                Ok(t) => t,
                                Err(e) => return e.into_compile_error().into(),
                            };
                            task.name = Some(f.sig.ident.clone());

                            let err = syn::Error::new(
                                f.span(),
                                "ids should be specified for all tasks or none",
                            )
                            .into_compile_error();

                            if task.id.is_none() {
                                if let Some(TaskIdState::Active) = task_ids {
                                    return err.into();
                                }

                                task.id = Some(ids);
                                ids += 1;
                                task_ids = Some(TaskIdState::Inactive);
                            } else {
                                if let Some(TaskIdState::Inactive) = task_ids {
                                    return err.into();
                                }
                                task_ids = Some(TaskIdState::Active);
                            }

                            tasks.push(task);
                        } else if attr
                            .path
                            .is_ident(&syn::Ident::new("init", proc_macro2::Span::call_site()))
                        {
                            if init_function.is_some() {
                                return syn::Error::new(
                                    f.span(),
                                    "only one init function should be defined",
                                )
                                .into_compile_error()
                                .into();
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

    let content = content.iter();
    let quanta_us = syn::LitInt::new(&args.quanta_us.to_string(), proc_macro2::Span::call_site());
    let ahb_freq = syn::LitInt::new(&args.ahb_freq.to_string(), proc_macro2::Span::call_site());
    let init = init::init(&tasks, args.scheduler, quanta_us, ahb_freq);

    (quote! {
        use ::chaos::scheduler::Scheduler;
        use ::chaos::os::Os;

        #(#content)*

        #init
    })
    .into()
}
