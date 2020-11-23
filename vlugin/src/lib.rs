use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

#[proc_macro_attribute]
pub fn vlugin(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as ItemFn);
    let name = func.sig.ident.clone();

    let handler = sync_req_handler(name);

    let plugin_def = quote! {
        #[no_mangle]
        pub extern "Rust" fn _request_handler() -> Box<dyn valor::RequestHandler> {
            sync_req_handler()
        }

        #handler

        #func
    };
    plugin_def.into()
}

fn sync_req_handler(handler: Ident) -> TokenStream2 {
    quote! {
        #[inline]
        fn sync_req_handler() -> Box<dyn valor::RequestHandler> {
            Box::new(|req| Box::pin(async { #handler(req) }) as valor::HandlerResponse)
        }
    }
}
