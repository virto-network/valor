use crate::JsRequest;
use async_trait::async_trait;
use js_sys::{Function, Object, Promise, Reflect};
use valor::{Plugin, Request, RequestHandler, Response};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;

pub(crate) struct Loader;

#[wasm_bindgen(
    inline_js = "export async function load(url) { return (await import(url)).handler }"
)]
extern "C" {
    #[wasm_bindgen(catch)]
    fn load(url: &str) -> Result<JsValue, JsValue>;
}

#[async_trait]
impl valor::Loader for Loader {
    async fn load(&self, plugin: &Plugin) -> Result<Box<dyn RequestHandler>, ()> {
        match plugin {
            Plugin::Web { name, url } => {
                let module = load(url.as_str()).await.map_err(|_| ())?;
                let module = module.unchecked_ref::<Object>();
                let handler = Reflect::get(&module, "handler").ok_or(());
                Ok(Box::new(JsHandler(handler.unchecked_into::<Function>())))
            }
            _ => unimplemented!(),
        }
    }
}

struct JsHandler(Function);

#[async_trait]
impl RequestHandler for JsHandler {
    async fn handle_request(&self, req: Request) -> Response {
        let promise = self.0.call1(JsValue::NULL, JsRequest::from(req)).unwrap();
        let response = JsFuture::from(Promise::resolve(promise)).await;
        response.into()
    }
}
