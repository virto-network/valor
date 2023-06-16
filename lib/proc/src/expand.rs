use quote::{quote, ToTokens, TokenStreamExt};

use crate::structs::{MethodData, ModuleData, KV};

impl ToTokens for KV {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.as_str().to_tokens(tokens);
        tokens.append(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone));
        self.1.as_str().to_tokens(tokens);
    }
}

impl ToTokens for MethodData {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let ident = &self.ident;
        let extensions = &self.extensions.0;

        let method_call = quote! {
            |request: &Request| -> Result<Response, ResponseError> {
                #ident(request)
            }
        };

        tokens.extend(quote! {
            Method {
                name: #name,
                call: Some(Box::new(#method_call)),
                extensions: {
                    let mut hm = BTreeMap::new();
                    #(
                        hm.insert(#extensions);
                    )*
                    hm
                },
            }
        });
    }
}

impl ToTokens for ModuleData {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let extensions = &self.extensions.0;
        let methods = &self.methods;

        tokens.extend(quote! {
            Module {
                name: #name,
                methods: vec![
                  #(
                    #methods,
                  )*
                ],
                extensions: {
                    let mut hm = BTreeMap::new();
                    #(
                        hm.insert(#extensions);
                    )*
                    hm
                },
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::structs::*;
    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens};

    #[test]
    fn test_method_data_to_tokens() {
        let method_tokens: TokenStream = quote! {
            #[valor::method("custom_method")]
            #[valor::extensions(http_verb = "GET", http_path = "/")]
            pub fn example_method(_req: &Request) -> Result<Response, ResponseError> {
                unimplemented!()
            }
        };

        let parsed_method = syn::parse2::<MethodData>(method_tokens).unwrap();
        let mut method_out: TokenStream = quote!();
        parsed_method.to_tokens(&mut method_out);

        let expected_out: TokenStream = quote! {
            Method {
                name: "custom_method",
                call: Some(Box::new(|request: &Request| -> Result<Response, ResponseError> {
                    example_method(request)
                })),
                extensions: {
                  let mut hm = BTreeMap::new();
                  hm.insert("http_verb", "GET");
                  hm.insert("http_path", "/");
                  hm
                },
            }
        };

        assert_eq!(method_out.to_string(), expected_out.to_string());
    }

    #[test]
    fn test_module_data_to_tokens() {
        let module_tokens: TokenStream = quote! {
            #[valor::module]
            pub mod test_module {
                #[valor::method("custom_method")]
                #[valor::extensions(http_verb = "GET", http_path = "/")]
                pub fn example_method<'a>(req: &Request<'a>) -> Result<Response, ResponseError> {
                    unimplemented!()
                }
            }
        };

        let parsed_module = syn::parse2::<ModuleData>(module_tokens).unwrap();
        let mut module_out: TokenStream = quote!();
        parsed_module.to_tokens(&mut module_out);

        let expected_out: TokenStream = quote! {
            Module {
                name: "test_module",
                methods: vec![
                    Method {
                        name: "custom_method",
                        call: Some(Box::new(|request: &Request| -> Result<Response, ResponseError> {
                            example_method(request)
                        })),
                        extensions: {
                          let mut hm = BTreeMap::new();
                          hm.insert("http_verb", "GET");
                          hm.insert("http_path", "/");
                          hm
                        },
                    },
                ],
                extensions: {
                  let mut hm = BTreeMap::new();
                  hm
                },
            }
        };

        println!("{}", &module_out.to_string());
        println!("{}", &expected_out.to_string());

        assert_eq!(module_out.to_string(), expected_out.to_string());
    }
}
