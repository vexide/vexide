use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Signature};

const NO_SYNC_ERR: &str = "The vexide entrypoint must be marked `async`.";
const NO_UNSAFE_ERR: &str = "The vexide entrypoint must be not marked `unsafe`.";
const WRONG_ARGS_ERR: &str = "The vexide entrypoint must take a single parameter of type `vexide_devices::peripherals::Peripherals`";

fn verify_function_sig(sig: &Signature) -> Result<(), syn::Error> {
    let mut error = None;

    if sig.asyncness.is_none() {
        let message = syn::Error::new_spanned(sig, NO_SYNC_ERR);
        error.replace(message);
    }
    if sig.unsafety.is_some() {
        let message = syn::Error::new_spanned(sig, NO_UNSAFE_ERR);
        match error {
            Some(ref mut e) => e.combine(message),
            None => {
                error.replace(message);
            }
        };
    }
    if sig.inputs.len() != 1 {
        let message = syn::Error::new_spanned(sig, WRONG_ARGS_ERR);
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

fn create_main_wrapper(inner: ItemFn) -> proc_macro2::TokenStream {
    match verify_function_sig(&inner.sig) {
        Ok(_) => {}
        Err(e) => return e.to_compile_error(),
    }
    let inner_ident = inner.sig.ident.clone();
    let ret_type = match &inner.sig.output {
        syn::ReturnType::Default => quote! { () },
        syn::ReturnType::Type(_, ty) => quote! { #ty },
    };

    quote! {
        #[no_mangle]
        extern "Rust" fn main() {
            #inner
            let termination: #ret_type = ::vexide::async_runtime::block_on(
                #inner_ident(::vexide::devices::peripherals::Peripherals::take().unwrap())
            );
            ::vexide::core::program::Termination::report(termination);
        }
    }
}

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    let main_fn = create_main_wrapper(item);

    quote! {
        #main_fn

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn wraps_main_fn() {
        let source = quote! {
            async fn main(_peripherals: Peripherals) {
                println!("Hello, world!");
            }
        };
        let input = syn::parse2::<ItemFn>(source).unwrap();
        let output = create_main_wrapper(input);
        assert_eq!(
            output.to_string(),
            quote! {
                #[no_mangle]
                extern "Rust" fn main() {
                    async fn main(_peripherals: Peripherals) {
                        println!("Hello, world!");
                    }
                    let termination: () = ::vexide::async_runtime::block_on(
                        main(::vexide::devices::peripherals::Peripherals::take().unwrap())
                    );
                    ::vexide::core::program::Termination::report(termination);
                }
            }
            .to_string()
        );
    }

    #[test]
    fn requires_async() {
        let source = quote! {
            fn main(_peripherals: Peripherals) {
                println!("Hello, world!");
            }
        };
        let input = syn::parse2::<ItemFn>(source).unwrap();
        let output = create_main_wrapper(input);
        assert!(output.to_string().contains(NO_SYNC_ERR));
    }

    #[test]
    fn requires_safe() {
        let source = quote! {
            async unsafe fn main(_peripherals: Peripherals) {
                println!("Hello, world!");
            }
        };
        let input = syn::parse2::<ItemFn>(source).unwrap();
        let output = create_main_wrapper(input);
        assert!(output.to_string().contains(NO_UNSAFE_ERR));
    }

    #[test]
    fn disallows_0_args() {
        let source = quote! {
            async fn main() {
                println!("Hello, world!");
            }
        };
        let input = syn::parse2::<ItemFn>(source).unwrap();
        let output = create_main_wrapper(input);
        assert!(output.to_string().contains(WRONG_ARGS_ERR));
    }

    #[test]
    fn disallows_2_args() {
        let source = quote! {
            async fn main(_peripherals: Peripherals, _other: Peripherals) {
                println!("Hello, world!");
            }
        };
        let input = syn::parse2::<ItemFn>(source).unwrap();
        let output = create_main_wrapper(input);
        assert!(output.to_string().contains(WRONG_ARGS_ERR));
    }
}
