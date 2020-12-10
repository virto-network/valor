use crate::{JsRequest, JsResponse};
use async_trait::async_trait;
use js_sys::{Function, Object, Promise, Reflect};
use valor::{Plugin, Request, RequestHandler, Response, StatusCode};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;

pub(crate) struct Loader;

#[wasm_bindgen(
    inline_js = "export async function load_handler(url) { return (await import(url)).handler }"
)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn load_handler(url: &str) -> Result<JsValue, JsValue>;
}

#[async_trait(?Send)]
impl valor::Loader for Loader {
    async fn load(&self, plugin: &Plugin) -> Result<Box<dyn RequestHandler>, ()> {
        match plugin {
            Plugin::Web { url, .. } => {
                let handler = load_handler(url.as_str()).await.map_err(|_| ())?;
                let handler = handler.unchecked_ref::<Object>();
                let handler = Reflect::get(&handler, &JsValue::from("handler")).map_err(|_| ())?;
                Ok(Box::new(JsHandler(handler.unchecked_into::<Function>())))
            }
            Plugin::Dummy => Ok(Box::new(|_| async { StatusCode::Ok.into() })),
            _ => Err(()),
        }
    }
}

struct JsHandler(Function);

#[async_trait(?Send)]
impl RequestHandler for JsHandler {
    async fn handle_request(&self, req: Request) -> Response {
        let promise = self.0.call1(&JsValue::NULL, &JsRequest::from(req)).unwrap();
        let response = JsFuture::from(Promise::resolve(&promise)).await.unwrap();
        let response = response.unchecked_into::<JsResponse>();
        response.into()
    }
}
