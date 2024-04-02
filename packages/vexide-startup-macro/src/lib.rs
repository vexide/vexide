use std::cell::OnceCell;

use proc_macro:: TokenStream;
use syn::{parse_macro_input, ItemFn, Signature};
use quote::quote;

fn verify_function_sig(sig: &Signature) -> Result<(), syn::Error> {
    let mut error = None;

    if sig.asyncness.is_none() {
        let message = syn::Error::new_spanned(&sig, "Function must be async");
        error.replace(message);
    }
    if sig.unsafety.is_some() {
        let message = syn::Error::new_spanned(&sig, "Function must be safe");
        match error {
            Some(ref mut e) => e.combine(message),
            None => { error.replace(message); },
        };
    }
    assert!(sig.inputs.is_empty());

    Ok(())
}

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    match verify_function_sig(&item.sig) {
        Ok(_) => {}
        Err(e) => return e.to_compile_error().into(),
    }

    let block = item.block;

    quote! {
        #[no_mangle]
        pub extern "C" fn main() {
            ::vexide_async::block_on(async #block);
        }

        #[no_mangle]
        #[link_section = ".boot"]
        unsafe extern "C" fn _entry() {
            unsafe {
                ::vexide_startup::program_entry()
            }
        }

        #[link_section = ".cold_magic"]
        #[used] // This is needed to prevent the linker from removing this object in release builds
        static COLD_HEADER: ::vexide_startup::ColdHeader = ::vexide_startup::ColdHeader::new(2, 0, 0);
    }.into()
}
