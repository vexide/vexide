use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Ident, LitBool, Result, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(banner);

    custom_keyword!(enabled);
    custom_keyword!(theme);

    custom_keyword!(code_sig);
}

#[derive(Clone)]
pub struct MacroOpts {
    pub banner_enabled: bool,
    pub banner_theme: Option<Ident>,
    pub code_sig: Option<Ident>,
}

impl Default for MacroOpts {
    fn default() -> Self {
        Self {
            banner_enabled: true,
            banner_theme: None,
            code_sig: None,
        }
    }
}

impl From<Attrs> for MacroOpts {
    fn from(value: Attrs) -> Self {
        let mut opts = Self::default();
        for attr in value.attr_list {
            match attr {
                Attribute::Banner(banner) => {
                    for attr in banner.attrs {
                        match attr {
                            BannerAttribute::Enabled(enabled) => {
                                opts.banner_enabled = enabled.as_bool();
                            }
                            BannerAttribute::Theme(theme) => {
                                opts.banner_theme = Some(theme.into_ident());
                            }
                        }
                    }
                }
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
    paren: token::Paren,
    attrs: Punctuated<BannerAttribute, Token![,]>,
}

impl Parse for Banner {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs;
        Ok(Self {
            token: input.parse()?,
            paren: parenthesized!(attrs in input),
            attrs: attrs.parse_terminated(BannerAttribute::parse, Token![,])?,
        })
    }
}

impl ToTokens for Banner {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token.to_tokens(tokens);
        self.paren.surround(tokens, |tokens| {
            self.attrs.to_tokens(tokens);
        });
    }
}

pub enum BannerAttribute {
    Enabled(BannerEnabled),
    Theme(BannerTheme),
}
impl Parse for BannerAttribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::enabled) {
            input.parse().map(BannerAttribute::Enabled)
        } else if lookahead.peek(kw::theme) {
            input.parse().map(BannerAttribute::Theme)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for BannerAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            BannerAttribute::Enabled(enabled) => enabled.to_tokens(tokens),
            BannerAttribute::Theme(theme) => theme.to_tokens(tokens),
        }
    }
}

pub struct BannerEnabled {
    token: kw::enabled,
    eq: Token![=],
    value: LitBool,
}
impl Parse for BannerEnabled {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            token: input.parse()?,
            eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl BannerEnabled {
    pub const fn as_bool(&self) -> bool {
        self.value.value
    }
}

impl ToTokens for BannerEnabled {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token.to_tokens(tokens);
        self.eq.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

pub struct BannerTheme {
    token: kw::theme,
    eq: Token![=],
    ident: Ident,
}

impl Parse for BannerTheme {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            token: input.parse()?,
            eq: input.parse()?,
            ident: input.parse()?,
        })
    }
}

impl BannerTheme {
    pub fn into_ident(self) -> Ident {
        self.ident
    }
}

impl ToTokens for BannerTheme {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token.to_tokens(tokens);
        self.eq.to_tokens(tokens);
        self.ident.to_tokens(tokens);
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
            banner(enabled = true, theme = THEME)
        };
        let input = syn::parse2::<Banner>(source).unwrap();
        assert!(input.attrs.len() == 2);
        assert!(matches!(
            input.attrs[0],
            BannerAttribute::Enabled(BannerEnabled {
                value: LitBool {
                    value: true,
                    span: _
                },
                ..
            })
        ));
        assert!(matches!(input.attrs[1], BannerAttribute::Theme(_)));
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
            banner(enabled = true, theme = THEME), code_sig = my_code_sig
        };
        let input = syn::parse2::<Attrs>(source).unwrap();
        assert_eq!(input.attr_list.len(), 2);
        let opts = MacroOpts::from(input);
        assert!(opts.banner_enabled);
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
        assert!(opts.banner_enabled);
        assert_eq!(opts.code_sig, None);

        let source = quote! {
            banner(enabled = false)
        };
        let opts = macro_opts_from(source);
        assert!(!opts.banner_enabled);
        assert_eq!(opts.code_sig, None);

        let source = quote! {
            code_sig = my_code_sig
        };
        let opts = macro_opts_from(source);
        assert!(opts.banner_enabled);
        assert_eq!(opts.code_sig.unwrap().to_string(), "my_code_sig");

        let source = quote! {
            banner(enabled = false), code_sig = my_code_sig
        };
        let opts = macro_opts_from(source);
        assert!(!opts.banner_enabled);
        assert_eq!(opts.code_sig.unwrap().to_string(), "my_code_sig");
    }
}
