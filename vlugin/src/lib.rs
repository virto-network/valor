use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn vlugin(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as ItemFn);
    let name = func.sig.ident.clone();

    let plugin_def = quote! {
        #[no_mangle]
        pub extern "Rust" fn _request_handler() -> Box<dyn valor::RequestHandler> {
            Box::new(|req| async { #name(req) }) as Box<dyn valor::RequestHandler>
        }

        #func
    };
    plugin_def.into()
}
