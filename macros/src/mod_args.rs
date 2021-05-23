use quote::ToTokens;

pub struct ModArgs {
    pub quanta_us: u32,
    pub scheduler: syn::Path,
    pub ahb_freq: u32,
}

impl syn::parse::Parse for ModArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut quanta_us = 10_000u32;
        let mut ahb_freq = 16u32 * 1_000_000;
        let mut scheduler = None;

        loop {
            if input.is_empty() {
                break;
            }

            let key = input.parse::<syn::Ident>()?;
            input.parse::<syn::token::Eq>()?;

            match key.to_token_stream().to_string().as_str() {
                "quanta_us" => quanta_us = input.parse::<syn::LitInt>()?.base10_parse()?,
                "scheduler" => scheduler = Some(input.parse::<syn::Path>()?),
                "ahb_freq" => ahb_freq = input.parse::<syn::LitInt>()?.base10_parse()?,
                _ => return Err(syn::Error::new(key.span(), format!("unknown attr"))),
            }

            if !input.peek(syn::token::Comma) {
                break;
            }
            input.parse::<syn::token::Comma>()?;
        }

        if scheduler.is_none() {
            return Err(syn::Error::new(input.span(), format!("scheduler is mandatory")));
        }

        Ok(ModArgs {quanta_us, scheduler: scheduler.unwrap(), ahb_freq})
    }
}
