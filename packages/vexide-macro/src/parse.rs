use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Result, Token,
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(banner);
    custom_keyword!(code_sig);
}

pub struct MacroOpts {
    pub banner: bool,
    pub code_sig: Option<Ident>,
}

impl Default for MacroOpts {
    fn default() -> Self {
        Self {
            banner: true,
            code_sig: None,
        }
    }
}

impl From<Attrs> for MacroOpts {
    fn from(value: Attrs) -> Self {
        let mut opts = Self::default();
        for attr in value.attr_list {
            match attr {
                Attribute::Banner(banner) => opts.banner = banner.as_bool(),
                Attribute::CodeSig(code_sig) => opts.code_sig = Some(code_sig.into_ident()),
            }
        }
        opts
    }
}

pub struct Attrs {
    attr_list: Punctuated<Attribute, Token![,]>,
}

impl Parse for Attrs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            attr_list: Punctuated::parse_terminated(input)?,
        })
    }
}

pub enum Attribute {
    Banner(Banner),
    CodeSig(CodeSig),
}

impl Parse for Attribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::banner) {
            input.parse().map(Attribute::Banner)
        } else if lookahead.peek(kw::code_sig) {
            input.parse().map(Attribute::CodeSig)
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct Banner {
    token: kw::banner,
    eq: Token![=],
    arg: syn::LitBool,
}

impl Banner {
    pub const fn as_bool(&self) -> bool {
        self.arg.value
    }
}

impl Parse for Banner {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            token: input.parse()?,
            eq: input.parse()?,
            arg: input.parse()?,
        })
    }
}

impl ToTokens for Banner {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token.to_tokens(tokens);
        self.eq.to_tokens(tokens);
        self.arg.to_tokens(tokens);
    }
}

pub struct CodeSig {
    token: kw::code_sig,
    eq: Token![=],
    ident: Ident,
}

impl CodeSig {
    pub fn into_ident(self) -> Ident {
        self.ident
    }
}

impl Parse for CodeSig {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            token: input.parse()?,
            eq: input.parse()?,
            ident: input.parse()?,
        })
    }
}

impl ToTokens for CodeSig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token.to_tokens(tokens);
        self.eq.to_tokens(tokens);
        self.ident.to_tokens(tokens);
    }
}

#[cfg(test)]
mod test {
    use quote::quote;
    use syn::Ident;

    use super::*;

    #[test]
    fn parses_banner_attribute() {
        let source = quote! {
            banner = true
        };
        let input = syn::parse2::<Banner>(source).unwrap();
        assert_eq!(input.as_bool(), true);
    }

    #[test]
    fn parses_code_sig_attribute() {
        let ident = Ident::new("my_code_sig", proc_macro2::Span::call_site());
        let source = quote! {
            code_sig = #ident
        };
        let input = syn::parse2::<CodeSig>(source).unwrap();
        assert_eq!(input.into_ident(), ident);
    }

    #[test]
    fn parses_attrs_into_macro_opts() {
        let source = quote! {
            banner = true, code_sig = my_code_sig
        };
        let input = syn::parse2::<Attrs>(source).unwrap();
        assert_eq!(input.attr_list.len(), 2);
        let opts = MacroOpts::from(input);
        assert_eq!(opts.banner, true);
        assert_eq!(opts.code_sig.unwrap().to_string(), "my_code_sig");
    }

    #[test]
    fn macro_opts_defaults_when_n_opts_missing() {
        fn macro_opts_from(source: TokenStream) -> MacroOpts {
            let input = syn::parse2::<Attrs>(source).unwrap();
            MacroOpts::from(input)
        }

        let source = quote! {};
        let opts = macro_opts_from(source);
        assert_eq!(opts.banner, true);
        assert_eq!(opts.code_sig, None);

        let source = quote! {
            banner = false
        };
        let opts = macro_opts_from(source);
        assert_eq!(opts.banner, false);
        assert_eq!(opts.code_sig, None);

        let source = quote! {
            code_sig = my_code_sig
        };
        let opts = macro_opts_from(source);
        assert_eq!(opts.banner, true);
        assert_eq!(opts.code_sig.unwrap().to_string(), "my_code_sig");

        let source = quote! {
            banner = false, code_sig = my_code_sig
        };
        let opts = macro_opts_from(source);
        assert_eq!(opts.banner, false);
        assert_eq!(opts.code_sig.unwrap().to_string(), "my_code_sig");
    }
}
