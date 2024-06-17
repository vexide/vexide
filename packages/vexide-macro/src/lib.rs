//! This crate provides a procedural macro for marking the entrypoint of a [vexide](https://vexide.dev) program.

use parse::{Attrs, MacroOpts};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Signature};

mod parse;

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

fn make_entrypoint(opts: MacroOpts) -> proc_macro2::TokenStream {
    let banner_arg = if opts.banner {
        quote! { true }
    } else {
        quote! { false }
    };
    let cold_header = if let Some(code_sig) = opts.code_sig {
        quote! { #code_sig }
    } else {
        quote! { ::vexide::startup::ColdHeader::new(2, 0, 0) }
    };

    quote! {
        const _: () = {
            #[no_mangle]
            #[link_section = ".boot"]
            unsafe extern "C" fn _entry() {
                unsafe {
                    ::vexide::startup::program_entry::<#banner_arg>()
                }
            }

            #[link_section = ".cold_magic"]
            #[used] // This is needed to prevent the linker from removing this object in release builds
            static COLD_HEADER: ::vexide::startup::ColdHeader = #cold_header;
        };
    }
}

/// Marks a function as the entrypoint for a vexide program. When the program is started,
/// the `main` function will be called with a single argument of type `Peripherals` which
/// allows access to device peripherals like motors, sensors, and the display.
///
/// The `main` function must be marked `async` and must not be marked `unsafe`. It may
/// return any type that implements `Termination`, which includes `()`, `!`, and `Result`.
///
/// # Parameters
///
/// The `main` attribute can be provided with parameters that alter the behavior of the program.
///
/// - `banner`: A boolean value that toggles the vexide startup banner printed over serial.
///   When `false`, the banner will be not displayed.
/// - `code_sig`: Allows using a custom `ColdHeader` struct to configure program behavior.
///
/// # Examples
///
/// The most basic usage of the `main` attribute is to mark an async function as the entrypoint
/// for a vexide program. The function must take a single argument of type `Peripherals`.
///
/// ```ignore
/// # #![no_std]
/// # #![no_main]
/// # use vexide::prelude::*;
/// # use core::fmt::Write;
/// #[vexide::main]
/// async fn main(mut peripherals: Peripherals) {
///     write!(peripherals.screen, "Hello, vexide!").unwrap();
/// }
/// ```
///
/// The `main` attribute can also be provided with parameters to customize the behavior of the program.
///
/// ```ignore
/// # #![no_std]
/// # #![no_main]
/// # use vexide::prelude::*;
/// #[vexide::main(banner = false)]
/// async fn main(_p: Peripherals) {
///    println!("This is the only serial output from this program!")
/// }
/// ```
///
/// A custom code signature may be used to further configure the behavior of the program.
///
/// ```ignore
/// # #![no_std]
/// # #![no_main]
/// # use vexide::prelude::*;
/// # use vexide::startup::ColdHeader;
/// static CODE_SIG: ColdHeader = ColdHeader::new(2, 0, 0);
/// #[vexide::main(code_sig = CODE_SIG)]
/// async fn main(_p: Peripherals) {
///    println!("Hello world!")
/// }
/// ```
#[proc_macro_attribute]
pub fn main(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    let attrs = parse_macro_input!(attrs as Attrs);
    let opts = MacroOpts::from(attrs);
    let main_fn = create_main_wrapper(item);
    let entrypoint = make_entrypoint(opts);

    quote! {
        #main_fn
        #entrypoint
    }
    .into()
}

#[cfg(test)]
mod test {
    use syn::Ident;

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
    fn toggles_banner_using_parsed_opts() {
        let entrypoint = make_entrypoint(MacroOpts {
            banner: false,
            code_sig: None,
        });
        assert!(entrypoint.to_string().contains("false"));
        assert!(!entrypoint.to_string().contains("true"));
        let entrypoint = make_entrypoint(MacroOpts {
            banner: true,
            code_sig: None,
        });
        assert!(entrypoint.to_string().contains("true"));
        assert!(!entrypoint.to_string().contains("false"));
    }

    #[test]
    fn uses_custom_code_sig_from_parsed_opts() {
        let entrypoint = make_entrypoint(MacroOpts {
            banner: false,
            code_sig: Some(Ident::new(
                "__custom_code_sig_ident__",
                proc_macro2::Span::call_site(),
            )),
        });
        println!("{}", entrypoint.to_string());
        assert!(entrypoint.to_string().contains(
            "static COLD_HEADER : :: vexide :: startup :: ColdHeader = __custom_code_sig_ident__ ;"
        ));
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
