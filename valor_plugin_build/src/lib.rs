use quote::{quote, ToTokens};
use std::{
    env,
    fs::{self, File},
    io::Read,
    path::Path,
};
use syn::{parse_quote, ReturnType};

pub fn build() {
    let src_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let mut lib_file = File::open(Path::new(&src_dir).join("src/lib.rs")).unwrap();
    let mut lib_content = String::new();
    lib_file.read_to_string(&mut lib_content).unwrap();
    let plugin = syn::parse_file(&lib_content).unwrap();

    let pub_fns = plugin
        .items
        .iter()
        .filter_map(|it| match it {
            syn::Item::Fn(f) => match f.vis {
                syn::Visibility::Public(_) => Some(f),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();

    let as_result = |it, none| match it {
        ReturnType::Default => parse_quote! { let _res = res; Ok(#none) },
        ReturnType::Type(_, ty) => {
            // Not very robust but "good enough" way to know if return type is a result
            if ty.to_token_stream().to_string().contains("Result") {
                quote!(res.map_err(|e| valor::Error::from(e)))
            } else {
                quote!(Ok(res))
            }
        }
    };

    let on_create = pub_fns
        .iter()
        .find_map(|f| {
            f.sig.ident.eq("on_create").then(|| {
                let create_res = as_result(f.sig.output.clone(), quote!(()));
                quote! {
                    let res = crate::on_create(&mut self.0).await;
                    #create_res
                }
            })
        })
        .unwrap_or_else(|| parse_quote!(Ok(())));

    let on_request = pub_fns
        .iter()
        .find(|f| &f.sig.ident == "on_request")
        .expect("request handler");

    let req_result = as_result(on_request.sig.output.clone(), quote!(valor::Answer::Pong));
    let req_args = if on_request.sig.inputs.len() == 2 {
        quote!(self.context(), req.into())
    } else {
        quote!(req.into())
    };

    let module = quote! {
        #[derive(Default)]
        pub struct Vlugin(valor::Context);

        #[valor::async_trait(?Send)]
        impl valor::Vlugin for Vlugin {
            async fn on_create(&mut self) -> Result<(), valor::Error> {
                #on_create
            }

            async fn on_msg(&self, req: valor::Message) -> Result<valor::Answer, valor::Error> {
                let res = crate::on_request(#req_args).await;
                #req_result.map(|res| valor::Answer::from(res))
            }

            fn context_mut(&mut self) -> &mut valor::Context {
                &mut self.0
            }
            fn context(&self) -> &valor::Context {
                &self.0
            }
        }
    };

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("vlugin.rs");
    fs::write(&dest_path, module.to_string()).unwrap();
}
