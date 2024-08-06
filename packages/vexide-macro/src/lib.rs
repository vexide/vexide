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

fn make_code_sig(opts: MacroOpts) -> proc_macro2::TokenStream {
    let sig = if let Some(code_sig) = opts.code_sig {
        quote! { #code_sig }
    } else {
        quote! {  ::vexide::core::program::CodeSignature::new(
            ::vexide::core::program::ProgramType::User,
            ::vexide::core::program::ProgramOwner::Partner,
            ::vexide::core::program::ProgramFlags::empty(),
        ) }
    };

    quote! {
        #[link_section = ".code_signature"]
        #[used] // This is needed to prevent the linker from removing this object in release builds
        static CODE_SIGNATURE: ::vexide::startup::CodeSignature = #sig;
    }
}

fn make_entrypoint(inner: &ItemFn, opts: MacroOpts) -> proc_macro2::TokenStream {
    match verify_function_sig(&inner.sig) {
        Ok(()) => {}
        Err(e) => return e.to_compile_error(),
    }
    let inner_ident = inner.sig.ident.clone();
    let ret_type = match &inner.sig.output {
        syn::ReturnType::Default => quote! { () },
        syn::ReturnType::Type(_, ty) => quote! { #ty },
    };

    let banner_theme = if let Some(theme) = opts.banner_theme {
        quote! { #theme }
    } else {
        quote! { ::vexide::startup::banner::themes::THEME_DEFAULT }
    };

    let banner_enabled = if opts.banner_enabled {
        quote! { true }
    } else {
        quote! { false }
    };

    quote! {
        #[no_mangle]
        unsafe extern "C" fn _start() -> ! {
            ::vexide::startup::startup::<#banner_enabled>(#banner_theme);

            #inner
            let termination: #ret_type = ::vexide::async_runtime::block_on(
                #inner_ident(::vexide::devices::peripherals::Peripherals::take().unwrap())
            );
            ::vexide::core::program::Termination::report(termination);
            ::vexide::core::program::exit();
        }
    }
}

/// vexide's entrypoint macro
///
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
///     write!(peripherals.display, "Hello, vexide!").unwrap();
/// }
/// ```
///
/// The `main` attribute can also be provided with parameters to customize the behavior of the program.
///
/// This includes disabling the banner or using a custom banner theme:
///
/// ```ignore
/// # #![no_std]
/// # #![no_main]
/// # use vexide::prelude::*;
/// #[vexide::main(banner(enabled = false))]
/// async fn main(_p: Peripherals) {
///    println!("This is the only serial output from this program!")
/// }
/// ```
#[proc_macro_attribute]
pub fn main(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let inner = parse_macro_input!(item as ItemFn);
    let attrs = parse_macro_input!(attrs as Attrs);
    let opts = MacroOpts::from(attrs);

    let entrypoint = make_entrypoint(&item, opts.clone());
    let code_signature = make_code_sig(opts);

    quote! {
        fn main() -> #ret_type {
            #inner

            ::vexide::runtime::__internal_entrypoint_task::<#banner_arg>();
            ::vexide::runtime::block_on(
                #inner_ident(::vexide::devices::peripherals::Peripherals::take().unwrap())
            )
        }
    }.into()
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

        let input = syn::parse2::<ItemFn>(source.clone()).unwrap();
        let output = make_entrypoint(&input, MacroOpts::default());

        assert_eq!(
            output.to_string(),
            quote! {
                #[no_mangle]
                unsafe extern "C" fn _start() -> ! {
                    ::vexide::startup::startup::<true>(::vexide::startup::banner::themes::THEME_DEFAULT);

                    #source

                    let termination: () = ::vexide::async_runtime::block_on(
                        main(::vexide::devices::peripherals::Peripherals::take().unwrap())
                    );

                    ::vexide::core::program::Termination::report(termination);
                    ::vexide::core::program::exit();
                }
            }
            .to_string()
        );
    }

    #[test]
    fn toggles_banner_using_parsed_opts() {
        let source = quote! {
            async fn main(_peripherals: Peripherals) {
                println!("Hello, world!");
            }
        };
        let input = syn::parse2::<ItemFn>(source.clone()).unwrap();
        let entrypoint = make_entrypoint(
            &input,
            MacroOpts {
                banner_enabled: false,
                banner_theme: None,
            },
        );
        assert!(entrypoint.to_string().contains("false"));
        assert!(!entrypoint.to_string().contains("true"));

        let entrypoint = make_entrypoint(
            &input,
            MacroOpts {
                banner_enabled: true,
                banner_theme: None,
            },
        );
        assert!(entrypoint.to_string().contains("true"));
        assert!(!entrypoint.to_string().contains("false"));
    }

    #[test]
    fn requires_async() {
        let source = quote! {
            fn main(_peripherals: Peripherals) {
                println!("Hello, world!");
            }
        };

        let input = syn::parse2::<ItemFn>(source.clone()).unwrap();
        let output = make_entrypoint(&input, MacroOpts::default());

        assert!(output.to_string().contains(NO_SYNC_ERR));
    }

    #[test]
    fn requires_safe() {
        let source = quote! {
            async unsafe fn main(_peripherals: Peripherals) {
                println!("Hello, world!");
            }
        };

        let input = syn::parse2::<ItemFn>(source.clone()).unwrap();
        let output = make_entrypoint(&input, MacroOpts::default());

        assert!(output.to_string().contains(NO_UNSAFE_ERR));
    }

    #[test]
    fn disallows_0_args() {
        let source = quote! {
            async fn main() {
                println!("Hello, world!");
            }
        };

        let input = syn::parse2::<ItemFn>(source.clone()).unwrap();
        let output = make_entrypoint(&input, MacroOpts::default());

        assert!(output.to_string().contains(WRONG_ARGS_ERR));
    }

    #[test]
    fn disallows_2_args() {
        let source = quote! {
            async fn main(_peripherals: Peripherals, _other: Peripherals) {
                println!("Hello, world!");
            }
        };

        let input = syn::parse2::<ItemFn>(source.clone()).unwrap();
        let output = make_entrypoint(&input, MacroOpts::default());

        assert!(output.to_string().contains(WRONG_ARGS_ERR));
    }
}
