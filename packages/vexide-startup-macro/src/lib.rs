use proc_macro:: TokenStream;
use syn::{parse_macro_input, ItemFn};
use quote::quote;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    assert_eq!(item.sig.ident, "main");
    assert!(item.sig.asyncness.is_some());
    assert!(item.sig.unsafety.is_none());
    assert!(item.sig.inputs.is_empty());

    let block = item.block;

    //TODO: Spin up the async executor
    quote! {
        #[no_mangle]
        pub extern "C" fn main() {
            #block
        }

        #[no_mangle]
        #[link_section = ".boot"]
        unsafe extern "C" fn _entry() {
            unsafe {
                ::vex_startup::program_entry()
            }
        }

        #[link_section = ".cold_magic"]
        #[used] // This is needed to prevent the linker from removing this object in release builds
        static COLD_HEADER: ::vex_startup::ColdHeader = ::vex_startup::::ColdHeader::new(2, 0, 0);
    }.into()
}
