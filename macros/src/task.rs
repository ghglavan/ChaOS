use quote::ToTokens;

pub struct Task {
    pub stack_size: u32,
    pub name: Option<syn::Ident>,
    pub privileged: bool,
    pub fp: bool,
}

impl Task {
    pub fn stack_name(&self) -> syn::Ident {
        quote::format_ident!("__{}_stack", self.name.clone().unwrap())
    }

    pub fn stack_size(&self) -> syn::LitInt {
        syn::LitInt::new(&self.stack_size.to_string(), proc_macro2::Span::call_site())
    }

    pub fn privileged(&self) -> syn::LitBool {
        syn::LitBool::new(self.privileged, proc_macro2::Span::call_site())
    }

    pub fn fp(&self) -> syn::LitBool {
        syn::LitBool::new(self.fp, proc_macro2::Span::call_site())
    }
}

impl syn::parse::Parse for Task {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = Task {
            stack_size: 0,
            name: None,
            privileged: false,
            fp: false,
        };

        let mut stack_size = None;

        loop {
            if input.is_empty() {
                break;
            }
            let key = input.parse::<syn::Ident>()?;

            match key.to_token_stream().to_string().as_str() {
                "stack_size" => {
                    input.parse::<syn::token::Eq>()?;
                    stack_size = Some(input.parse::<syn::LitInt>()?.base10_parse::<u32>()?)
                }
                "privileged" => args.privileged = true,
                "fp" => args.fp = true,
                _ => return Err(syn::Error::new(key.span(), format!("unknown attr"))),
            }

            if !input.peek(syn::token::Comma) {
                break;
            }
        }

        args.stack_size =
            stack_size.ok_or(syn::Error::new(input.span(), "stack_size is mandatory"))?;

        Ok(args)
    }
}
