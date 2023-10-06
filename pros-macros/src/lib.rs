use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl};

/// Configure a struct implementing [`pros::Robot`] to receive calls
/// from the PROS kernel, such as `opcontrol` and `autonomous`.
/// There can only be one `#[robot]` usage per program.
///
/// ```
/// # use pros::prelude::*;
/// pub struct ClawBot {
///     claw_motor: Motor,
/// }
///
/// #[robot]
/// impl Robot for ClawBot {
///    /// Runs immediately after the program starts.
///    fn init() -> pros::Result<Self> {
///        Ok(Self {
///           claw_motor: Motor::new(1, BrakeMode::Brake)?,
///       })
///    }
///
///   /// Runs when the robot is enabled (or immediately
///   /// if you aren't in a competition).
///   fn opcontrol(&mut self) -> pros::Result {
///      todo!()
///   }
/// }
/// ```
///
/// # Initialization
///
/// The `init` function must return an instance of the struct implementing
/// the Robot trait. If you do not want to implement the `init` function,
/// you can implement or derive [`Default`] on your struct, which will be
/// used instead. Later in the program, you can access the robot instance
/// through the `&mut self` parameter of the Robot methods.
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

        #[doc(hidden)]
        static mut __PROS_ROBOT: Option<#rbt> = None;

        #[doc(hidden)]
        #[export_name = "opcontrol"]
        extern "C" fn __pros_opcontrol() {
            ::pros::Robot::opcontrol(unsafe {
                __PROS_ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            })
            .unwrap();
        }

        #[doc(hidden)]
        #[export_name = "autonomous"]
        extern "C" fn __pros__autonomous() {
            ::pros::Robot::auto(unsafe {
                __PROS_ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before auto")
            })
            .unwrap();
        }

        #[doc(hidden)]
        #[export_name = "initialize"]
        extern "C" fn __pros_initialize() {
            unsafe {
                ::pros::__pros_sys::lcd_initialize();
            }
            unsafe {
                __PROS_ROBOT = Some(::pros::Robot::init().unwrap());
            }
        }

        #[doc(hidden)]
        #[export_name = "disabled"]
        extern "C" fn __pros_disabled() {
            ::pros::Robot::disabled(unsafe {
                __PROS_ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before disabled")
            })
            .unwrap();
        }


        #[doc(hidden)]
        #[export_name = "competition_initialize"]
        extern "C" fn __pros_competition_initialize() {
            ::pros::Robot::comp_init(unsafe {
                __PROS_ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before comp_init")
            })
            .unwrap();
        }
    };

    TokenStream::from(expanded)
}
