use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Expr, ItemFn, ItemMod, Lit, LitStr, Meta, Token,
};

use crate::structs::*;

impl Parse for KV {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(Meta::NameValue(kv)) = input.parse() {
            if let Some(k) = kv.path.segments.first() {
                if let Expr::Lit(l) = kv.value {
                    if let Lit::Str(v) = l.lit {
                        Ok(KV(k.ident.to_string(), v.value()))
                    } else {
                        Err(syn::Error::new(
                            l.span(),
                            "`value` should be a literal string",
                        ))
                    }
                } else {
                    Err(syn::Error::new(
                        kv.value.span(),
                        "`value` should be a literal (i.e. 1, \"a\"",
                    ))
                }
            } else {
                Err(syn::Error::new(
                    kv.path.span(),
                    "`key` should be a variable name (i.e. http_path)",
                ))
            }
        } else {
            Err(syn::Error::new(
                input.span(),
                "Variables inside #[valor::extensions] should be key-value items",
            ))
        }
    }
}

impl Parse for Extensions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut ext = Extensions::default();

        loop {
            if input.is_empty() {
                break;
            }

            let kv = input.parse::<KV>()?;
            ext.0.push(kv);

            if input.is_empty() {
                break;
            }

            input.parse::<Token!(,)>()?;
        }

        Ok(ext)
    }
}

impl Parse for MethodData {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item: ItemFn = input.parse()?;

        let extensions = item
            .attrs
            .iter()
            .find(|attr| attr.path().segments.iter().any(|s| s.ident == "extensions"))
            .and_then(|attr| {
                Some(
                    attr.parse_args_with(Extensions::parse)
                        .unwrap_or(Extensions::default()),
                )
            })
            .unwrap_or(Extensions::default());

        let default_method_name = item.sig.ident.to_string();

        let name = item
            .attrs
            .iter()
            .find(|attr| attr.path().segments.iter().any(|s| s.ident == "method"))
            .and_then(|attr| attr.parse_args::<LitStr>().ok().map(|lit| lit.value()))
            .unwrap_or(default_method_name);

        Ok(Self {
            name,
            ident: item.sig.ident,
            extensions,
        })
    }
}

impl Parse for ModuleData {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item: ItemMod = input.parse()?;
        let ident = item.ident;
        let name = ident.to_string();

        let extensions = item
            .attrs
            .iter()
            .find(|attr| attr.path().segments.iter().any(|s| s.ident == "extensions"))
            .and_then(|attr| {
                Some(
                    attr.parse_args_with(Extensions::parse)
                        .unwrap_or(Extensions::default()),
                )
            })
            .unwrap_or(Extensions::default());

        let content = match item.content {
            Some((_, items)) => items,
            None => return Err(input.error("Module must have content")),
        };

        let methods = content
            .into_iter()
            .filter_map(|item| {
                if let syn::Item::Fn(item_fn) = item {
                    Some(syn::parse2(item_fn.to_token_stream()))
                } else {
                    None
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            name,
            ident,
            extensions,
            methods,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::parse::Parser;

    #[test]
    fn test_method_data_parse() {
        let method_tokens: TokenStream = quote! {
            #[valor::method("custom_method")]
            #[valor::extensions(http_verb = "GET", http_path = "/")]
            pub fn example_method(_req: &Request) -> Result<Response, ResponseError> {
                unimplemented!()
            }
        };

        let parser = MethodData::parse;
        let parsed = parser.parse2(method_tokens).unwrap();

        assert_eq!(parsed.name, "custom_method".to_string());

        assert_eq!(parsed.extensions.0[0].0, "http_verb".to_string());
        assert_eq!(parsed.extensions.0[0].1, "GET".to_string());

        assert_eq!(parsed.extensions.0[1].0, "http_path".to_string());
        assert_eq!(parsed.extensions.0[1].1, "/".to_string());
    }

    #[test]
    fn test_method_data_parse_defaults() {
        let method_tokens: TokenStream = quote! {
            #[valor::method]
            pub fn example_method(_req: &Request) -> Result<Response, ResponseError> {
                unimplemented!()
            }
        };

        let parser = MethodData::parse;
        let parsed = parser.parse2(method_tokens).unwrap();

        assert_eq!(parsed.name, "example_method".to_string());
        assert_eq!(parsed.extensions.0.len(), 0);
    }

    #[test]
    fn test_module_data_parse() {
        let module_tokens: TokenStream = quote! {
            #[valor::module]
            #[valor::extensions(http_path = "/")]
            pub mod test_module {
                #[valor::method("custom_method")]
                #[valor::extensions(http_verb = "GET", http_path = "/")]
                pub fn example_method(_req: &Request) -> Result<Response, ResponseError> {
                    unimplemented!()
                }
            }
        };

        let parser = ModuleData::parse;
        let parsed = parser.parse2(module_tokens).unwrap();

        assert_eq!(parsed.name, "test_module".to_string());

        assert_eq!(parsed.extensions.0[0].0, "http_path".to_string());
        assert_eq!(parsed.extensions.0[0].1, "/".to_string());

        assert_eq!(parsed.methods[0].name, "custom_method".to_string());
        assert_eq!(parsed.methods[0].extensions.0[0].0, "http_verb".to_string());
        assert_eq!(parsed.methods[0].extensions.0[0].1, "GET".to_string());

        assert_eq!(parsed.methods[0].extensions.0[1].0, "http_path".to_string());
        assert_eq!(parsed.methods[0].extensions.0[1].1, "/".to_string());
    }
}
