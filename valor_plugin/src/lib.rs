//! Valor "vlugin" is a macro that creates a struct implementing the
//! Vlugin trait using

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Error, ItemFn};

/// vlugin
#[proc_macro_attribute]
pub fn vlugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: proc_macro2::TokenStream = item.into();

    if let Ok(func) = syn::parse2::<ItemFn>(item.clone()) {
        let is_pub = |f: &ItemFn| match f.vis {
            syn::Visibility::Public(_) => true,
            _ => false,
        };
        if func.sig.asyncness.is_none() || !is_pub(&func) {
            return Error::new(
                func.sig.fn_token.span,
                "Function neeeds to be \"pub async\"",
            )
            .to_compile_error()
            .into();
        }

        let name = &func.sig.ident;
        match name.to_string().as_str() {
            "on_create" | "on_request" => {}
            _ => {
                return Error::new(
                    name.span(),
                    "Function should either be named \"on_create\" or \"on_request\"",
                )
                .to_compile_error()
                .into()
            }
        };

        // NOTE currently relying on a build script to generate the Vlugin struct implementation
        // Once custom inner attributes are supported(https://github.com/rust-lang/rust/issues/54726)
        // we can do all the necessary parsing of the file within the macro.
        let module: TokenStream2 = quote! {
            mod v {
                include!(concat!(env!("OUT_DIR"), "/vlugin.rs"));
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub extern "Rust" fn instantiate_vlugin() -> impl valor::Vlugin {
                v::Vlugin::default()
            }

            #item
        };
        module.into()
    } else {
        Error::new(
            proc_macro2::Span::mixed_site(),
            "Can only annotate functions",
        )
        .to_compile_error()
        .into()
    }
}
