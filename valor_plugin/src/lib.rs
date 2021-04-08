//! Valor plugin

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse_quote, Error, ItemFn, ItemStruct};

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
    use syn::ReturnType;
    let name = &item.sig.ident;
    let return_ty = item.sig.output.clone();

    if item.sig.asyncness.is_none() {
        return Error::new(item.sig.fn_token.span, "Function neeeds to be async")
            .to_compile_error()
            .into();
    }

    let ret = match return_ty {
        ReturnType::Default => parse_quote! { Ok(valor::Output::None) },
        ReturnType::Type(_, ty) => {
            // Not very robust but "good enough" way to know if return type is a result
            if ty.to_token_stream().to_string().contains("Result") {
                quote!(res)
            } else {
                quote!(Ok(res))
            }
        }
    };

    match name.to_string().as_str() {
        "on_request" => {
            quote! {
                #[cfg(target_arch = "wasm32")]
                use valor::web::{web_sys, wasm_bindgen, wasm_bindgen_futures, into_request, into_js_response};
                #[cfg(target_arch = "wasm32")]
                use wasm_bindgen::prelude::*;

                pub struct Vlugin;

                #[valor::async_trait(?Send)]
                impl valor::Handler for Vlugin {
                    async fn on_msg(&self, req: valor::Message) -> Result<valor::Output, valor::Error> {
                        let res = $crate::on_request(req.into()).await;
                        #ret.map(|req| valor::Output::from(req))
                    }
                }

                #[cfg(not(target_arch = "wasm32"))]
                pub extern "Rust" fn instantiate_vlugin() -> impl valor::Handler {
                    Vlugin
                }

                #[cfg(target_arch = "wasm32")]
                #[wasm_bindgen]
                pub async fn handler(req: web_sys::Request) -> web_sys::Response {
                    let v = Vlugin;
                    //TODO how to handle result in Web
                    let res = v.on_msg(into_request(req).await.into()).await.unwrap();
                    into_js_response(res.into()).await
                }

                #item
            }
        }
        "on_create" => quote! {},
        _ => Error::new(
            name.span(),
            "Function should be named \"on_create\" or \"on_request\"",
        )
        .to_compile_error()
        .into(),
    }
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

    plugin_def
}
