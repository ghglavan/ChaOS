use quote::ToTokens;

pub struct Task {
    pub stack_size: u32,
    pub id: Option<u32>,
    pub prio: u32,
    pub name: Option<syn::Ident>,
    pub privileged: bool,
    pub fp: bool,
    pub disabled: bool,
}

impl Task {
    pub fn stack_name(&self) -> syn::Ident {
        let mut name_upper = self.name.clone().unwrap().to_string();
        name_upper.make_ascii_uppercase();
        quote::format_ident!("__{}_STACK", name_upper)
    }

    pub fn id(&self) -> syn::LitInt {
        syn::LitInt::new(
            &self.id.unwrap().to_string(),
            proc_macro2::Span::call_site(),
        )
    }

    pub fn prio(&self) -> syn::LitInt {
        syn::LitInt::new(&self.prio.to_string(), proc_macro2::Span::call_site())
    }

    pub fn disabled(&self) -> syn::LitBool {
        syn::LitBool::new(self.disabled, proc_macro2::Span::call_site())
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
            id: None,
            prio: 0,
            name: None,
            privileged: false,
            fp: false,
            disabled: false,
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
                "id" => {
                    input.parse::<syn::token::Eq>()?;
                    args.id = Some(input.parse::<syn::LitInt>()?.base10_parse::<u32>()?)
                }
                "prio" => {
                    input.parse::<syn::token::Eq>()?;
                    args.prio = input.parse::<syn::LitInt>()?.base10_parse::<u32>()?
                }
                "disabled" => args.disabled = true,
                _ => return Err(syn::Error::new(key.span(), "unknown attr".to_string())),
            }

            if !input.peek(syn::token::Comma) {
                break;
            }
        }

        args.stack_size =
            stack_size.ok_or_else(|| syn::Error::new(input.span(), "stack_size is mandatory"))?;

        Ok(args)
    }
}
