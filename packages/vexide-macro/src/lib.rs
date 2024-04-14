use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, Signature, Type};

fn verify_function_sig(sig: &Signature) -> Result<(), syn::Error> {
    let mut error = None;

    if sig.asyncness.is_none() {
        let message = syn::Error::new_spanned(sig, "Function must be async");
        error.replace(message);
    }
    if sig.unsafety.is_some() {
        let message = syn::Error::new_spanned(sig, "Function must be safe");
        match error {
            Some(ref mut e) => e.combine(message),
            None => {
                error.replace(message);
            }
        };
    }
    if sig.inputs.len() != 1 {
        let message = syn::Error::new_spanned(
            sig,
            "Function must take a `vexide_devices::peripherals::Peripherals`",
        );
        match error {
            Some(ref mut e) => e.combine(message),
            None => {
                error.replace(message);
            }
        };
    }

    match error {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    match verify_function_sig(&item.sig) {
        Ok(_) => {}
        Err(e) => return e.to_compile_error().into(),
    }
    let err = syn::Error::new_spanned(&item.sig, "Function must take a `Peripherals`")
        .to_compile_error()
        .into();
    let FnArg::Typed(peripherals_arg) = item.sig.inputs.first().unwrap() else {
        return err;
    };
    let Pat::Ident(ref peripherals_pat) = *peripherals_arg.pat else {
        return err;
    };
    let peripherals_ident = &peripherals_pat.ident;
    let ret_type = match &item.sig.output {
        syn::ReturnType::Default => quote! { () },
        syn::ReturnType::Type(_, ty) => quote! { #ty },
    };

    let block = item.block;

    quote! {
        #[no_mangle]
        extern "Rust" fn main() {
            let #peripherals_ident = ::vexide::devices::peripherals::Peripherals::take().unwrap();

            let termination: #ret_type = ::vexide::async_runtime::block_on(async #block);
            ::vexide::core::program::Termination::report(termination);
        }

        #[no_mangle]
        #[link_section = ".boot"]
        unsafe extern "C" fn _entry() {
            unsafe {
                ::vexide::startup::program_entry()
            }
        }

        #[link_section = ".cold_magic"]
        #[used] // This is needed to prevent the linker from removing this object in release builds
        static COLD_HEADER: ::vexide::startup::ColdHeader = ::vexide::startup::ColdHeader::new(2, 0, 0);
    }.into()
}
