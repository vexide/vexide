use proc_macro::{self, TokenStream};
use quote::quote;

#[proc_macro_attribute]
pub fn auto(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = syn::parse(item).unwrap();
    let ident = syn::Ident::new("autonomous", proc_macro::Span::call_site().into());

    extern_c_wrapper(function, ident)
}

#[proc_macro_attribute]
pub fn init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = syn::parse(item).unwrap();
    let ident = syn::Ident::new("initialize", proc_macro::Span::call_site().into());

    extern_c_wrapper(function, ident)
}

#[proc_macro_attribute]
pub fn disabled(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = syn::parse(item).unwrap();
    let ident = syn::Ident::new("disabled", proc_macro::Span::call_site().into());

    extern_c_wrapper(function, ident)
}

#[proc_macro_attribute]
pub fn comp_init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = syn::parse(item).unwrap();
    let ident = syn::Ident::new(
        "competition_initialize",
        proc_macro::Span::call_site().into(),
    );

    extern_c_wrapper(function, ident)
}

#[proc_macro_attribute]
pub fn opcontrol(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = syn::parse(item).unwrap();
    let ident = syn::Ident::new("opcontrol", proc_macro::Span::call_site().into());

    extern_c_wrapper(function, ident)
}

fn extern_c_wrapper(function: syn::ItemFn, ident: syn::Ident) -> TokenStream {
    let block = function.block;
    let gen: TokenStream = quote! {
        #[no_mangle]
        pub extern "C" fn #ident () #block
    }
    .into();

    gen
}
