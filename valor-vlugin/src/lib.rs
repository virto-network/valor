//! Valor vlugin

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Error, ItemFn, ItemStruct};

/// vlugin
#[proc_macro_attribute]
pub fn vlugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ts2: proc_macro2::TokenStream = item.into();

    if let Ok(rslt) = syn::parse2::<ItemFn>(ts2.clone()) {
        return handle_item_fn(rslt).into();
    }

    if let Ok(rslt) = syn::parse2::<ItemStruct>(ts2) {
        return handle_item_struct(rslt).into();
    }

    Error::new(
        proc_macro2::Span::mixed_site(),
        "Only functions and structures that implement `RequestHandler` are currently supported",
    )
    .to_compile_error()
    .into()
}

fn handle_item_fn(item: ItemFn) -> TokenStream2 {
    let name = item.sig.ident.clone();

    let plugin_def = quote! {
        /// Handler
        #[no_mangle]
        pub extern "Rust" fn get_request_handler() -> Box<dyn valor::RequestHandler> {
            Box::new(|req| Box::pin(async { #name(req) }) as valor::HandlerResponse)
        }

        #item
    };

    plugin_def.into()
}

fn handle_item_struct(item: ItemStruct) -> TokenStream2 {
    let name = &item.ident;

    let plugin_def = quote! {
        /// Handler
        #[no_mangle]
        pub extern "Rust" fn get_request_handler() -> Box<dyn valor::RequestHandler> {
            Box::new(#name::default())
        }

        #item
    };

    plugin_def.into()
}
