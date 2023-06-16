extern crate alloc;

pub(crate) mod expand;
pub(crate) mod parse;
pub(crate) mod structs;

use proc_macro::TokenStream;
use quote::quote;
use structs::ModuleData;

/// The `module` attribute macro for the `valor` crate.
///
/// This macro marks a module to be included in the `valor` runtime.
/// It processes the functions within the module, generating a vector of `Method` structures.
/// Each `Method` represents a function within the module, including its name, the function call as a closure,
/// and any associated extensions defined by `valor::extensions`.
///
/// # Example
/// ```ignore
/// use valor_proc::{module, method, extensions};
/// use valor_core::primitives::{Request, Response, ResponseError};
///
/// #[module("a_module")]
/// pub mod a_module {
///   #[method("a_method")]
///   #[extensions(http_verb = "GET", http_path = "/")]
///   pub fn a_method<'a> (request: &Request<'a>) -> Result<Response, ResponseError> {
///     unimplemented!()
///   }
/// }
/// ```
/// In the example above, the `module` macro will generate a `Module` structure for `a_module`
/// and a `Method` for `a_method` with the extensions `http_verb` and `http_path`.
#[proc_macro_attribute]
pub fn module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mod_item: proc_macro2::TokenStream = item.clone().into();

    match syn::parse::<ModuleData>(item) {
        Ok(module) => {
            let mod_ident = &module.ident;

            quote! {
                #[cfg(no_std)]
                extern crate alloc;
                #[cfg(no_std)]
                use alloc::collections::BTreeMap;

                #[cfg(not(no_std))]
                use std::collections::BTreeMap;

                use self::#mod_ident::*;
                use valor::{
                    lazy_static,
                    structures::{Module, Method}
                };

                #mod_item

                lazy_static! {
                    static ref MODULE: Module<'static> = #module;
                }

                #[cfg(not(target_arch = "wasm32"))]
                #[ctor::ctor]
                fn init() {
                    valor::registry::add_module(&MODULE);
                }

                #[cfg(target_arch = "wasm32")]
                #[no_mangle]
                pub exterrn "C" fn __valor_export_module() -> (*const u8, usize) {
                    valor::interop::export_module(&*MODULE)
                }

                #[cfg(target_arch = "wasm32")]
                #[no_mangle]
                pub exterrn "C" fn __valor_make_call<'a>(method_name: &str, request: &str) -> (*const u8, usize) {
                    valor::interop::make_call(&*MODULE, method_name, request)
                }

                pub fn main () {
                    valor::interop::handle_command(&*MODULE);
                }
            }
            .into()
        }
        Err(error) => error.to_compile_error().into(),
    }
}

/// The `valor::method` attribute macro is used to mark methods within a
/// `valor::module`. The `module` macro uses this to recognize the methods.
/// This macro currently doesn't perform any transformation or generation of code.
#[proc_macro_attribute]
pub fn method(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Return the input item as is
    item
}

/// The `valor::extensions` attribute macro is used to provide metadata for
/// methods within a `valor::module`. The `module` macro uses this to extract the
/// extensions. This macro currently doesn't perform any transformation or generation
/// of code.
#[proc_macro_attribute]
pub fn extensions(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Return the input item as is
    item
}
