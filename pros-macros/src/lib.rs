use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl};

#[proc_macro_attribute]
pub fn robot(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut robot_impl = parse_macro_input!(input as ItemImpl);

    let has_init_fn = robot_impl.items.iter().any(|item| match item {
        syn::ImplItem::Fn(f) => f.sig.ident == "init",
        _ => false,
    });
    if !has_init_fn {
        let init_fn: TokenStream = quote! {
            fn init() -> ::pros::Result<Self>
            where
                Self: Sized,
            {
                Ok(::core::default::Default::default())
            }
        }
        .into();
        robot_impl
            .items
            .push(syn::ImplItem::Fn(parse_macro_input!(init_fn)));
    }

    let rbt = &robot_impl.self_ty;
    let expanded = quote::quote! {
        #robot_impl

        mod __pros_internals {
        use super::*;
        static mut ROBOT: Option<#rbt> = None;

        #[no_mangle]
        extern "C" fn opcontrol() {
        #rbt::opcontrol(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            })
            .unwrap();
        }

        #[no_mangle]
        extern "C" fn autonomous() {
            #rbt::auto(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before auto")
            })
            .unwrap();
        }

        #[no_mangle]
        extern "C" fn initialize() {
            unsafe {
                ::pros::__pros_sys::lcd_initialize();
            }
            unsafe {
                ROBOT = Some(#rbt::init().unwrap());
            }
        }

        #[no_mangle]
        extern "C" fn disabled() {
            #rbt::disabled(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before disabled")
            })
            .unwrap();
        }

        #[no_mangle]
        extern "C" fn competition_initialize() {
            #rbt::comp_init(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before comp_init")
            })
            .unwrap();
        }
        }
    };

    TokenStream::from(expanded)
}
