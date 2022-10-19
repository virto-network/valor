use async_trait::async_trait;
use js_sys::{Function, Promise};
use log::{debug, warn};
use valor::{
    web::{into_js_request, into_response},
    Request, RequestHandler, Response, VluginType,
};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::Response as JsResponse;

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
    type Handler = JsHandler;

    async fn load(&self, plugin: &VluginType) -> valor::LoadResult<Self> {
        match plugin {
            VluginType::Web { url, name, .. } => {
                debug!("loading plugin {} from {}", name, url);
                let handler = load_handler(url.as_str()).await.map_err(|_| {
                    warn!("failed loading {}", name);
                    valor::LoadError::NotFound
                })?;
                let handler = handler.dyn_into::<Function>().map_err(|_| {
                    warn!("{} doesn't export handler", name);
                    valor::LoadError::BadFormat
                })?;
                Ok(JsHandler(handler))
            }
            _ => Err(valor::LoadError::NotSupported),
        }
    }
}

pub(crate) struct JsHandler(Function);

#[async_trait(?Send)]
impl RequestHandler for JsHandler {
    async fn on_request(&self, req: Request) -> valor::http::Result<Response> {
        let (req, _body) = into_js_request(req).await;
        let promise = self.0.call1(&JsValue::NULL, &req).unwrap();
        let response = JsFuture::from(Promise::resolve(&promise)).await.unwrap();
        let response = response.unchecked_into::<JsResponse>();
        into_response(response).await
    }
}
