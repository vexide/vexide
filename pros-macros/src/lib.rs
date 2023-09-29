use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, ItemStruct};

/*
#[macro_export]
macro_rules! robot {
    ($rbt:ty) => {
        pub static mut ROBOT: Option<$rbt> = None;

        #[no_mangle]
        extern "C" fn opcontrol() {
            <$rbt as $crate::Robot>::opcontrol(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            })
            .unwrap();
        }

        #[no_mangle]
        extern "C" fn autonomous() {
            <$rbt as $crate::Robot>::auto(unsafe {
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
                ROBOT = Some(<$rbt as $crate::Robot>::init().unwrap());
            }
        }

        #[no_mangle]
        extern "C" fn disabled() {
            <$rbt as $crate::Robot>::disabled(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before disabled")
            })
            .unwrap();
        }

        #[no_mangle]
        extern "C" fn competition_initialize() {
            <$rbt as $crate::Robot>::comp_init(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before comp_init")
            })
            .unwrap();
        }
    };
}
*/

#[proc_macro_attribute]
pub fn robot(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut default_init = false;
    let robot_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("default") {
            default_init = true;
            Ok(())
        } else {
            Err(meta.error("unsupported robot property"))
        }
    });
    parse_macro_input!(args with robot_parser);

    let mut robot_impl = parse_macro_input!(input as ItemImpl);

    if default_init {
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
        pub static mut ROBOT: Option<#rbt> = None;

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
    };

    TokenStream::from(expanded)
}
