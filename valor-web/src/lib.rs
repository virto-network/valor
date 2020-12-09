//! Valor web

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

use js_sys::{Array, ArrayBuffer, Object, Reflect, Uint8Array};
use loader::Loader;
use std::rc::Rc;
use std::sync::Arc;
use valor::{Handler, Request, Response, Url};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, BroadcastChannel, MessageEvent};

mod loader;

#[wasm_bindgen]
extern "C" {
    type JsResponse;
    #[wasm_bindgen(method, getter)]
    fn status(this: &JsResponse) -> u16;
    #[wasm_bindgen(method, getter)]
    fn res_headers(this: &JsResponse) -> Vec<JsValue>;
    #[wasm_bindgen(method, getter)]
    fn res_body(this: &JsResponse) -> ArrayBuffer;
}

#[wasm_bindgen]
extern "C" {
    type JsRequest;
    #[wasm_bindgen(method, getter)]
    fn method(this: &JsRequest) -> String;
    #[wasm_bindgen(method, getter)]
    fn url(this: &JsRequest) -> String;
    #[wasm_bindgen(method, getter)]
    fn headers(this: &JsRequest) -> Vec<JsValue>;
    #[wasm_bindgen(method, getter)]
    fn body(this: &JsRequest) -> ArrayBuffer;
}

impl From<Request> for JsRequest {
    fn from(req: Request) -> Self {
        // TODO
        let request = Object::new();
        Reflect::set(
            &request,
            &JsValue::from("method"),
            &JsValue::from(req.method().as_ref()),
        )
        .unwrap();
        request.unchecked_into::<JsRequest>()
    }
}

impl From<JsRequest> for Request {
    fn from(req: JsRequest) -> Self {
        let method = req.method().parse().expect("valid method");
        let url = Url::parse(&req.url()).expect("valid url");
        let body = Uint8Array::new(&req.body()).to_vec();
        let mut request = Request::new(method, url);
        for h in req.headers() {
            let h = h.unchecked_ref::<Array>();
            let name = h.get(0).as_string().unwrap();
            let value = h.get(1).as_string().unwrap();
            request.insert_header(&*name, &*value);
        }
        request.set_body(body);
        request
    }
}

impl From<JsResponse> for Response {
    fn from(res: JsResponse) -> Self {
        let status = res.status();
        let _method = res.res_body();
        let body = Uint8Array::new(&res.res_body()).to_vec();
        // TODO headers
        let mut response = Response::new(status);
        response.set_body(body);
        response
    }
}

/// Run
#[wasm_bindgen(start)]
pub async fn run() -> Result<(), JsValue> {
    init_log();
    load_service_worker("sw.js")?;

    let handler = Handler::new(Arc::new(Loader));

    let req_channel = BroadcastChannel::new("req_channel")?;
    let res_channel = Rc::new(BroadcastChannel::new("res_channel")?);

    // handle incoming requests from service worker
    // we get a JS Object with request data on the first BroadcastChannel
    // and must send response data through the sencond one in a timely
    // manner or the service worker will respond with a timeout
    let on_msg = Closure::wrap(Box::new(move |e: MessageEvent| {
        let req = e.data();
        let req = req.unchecked_into::<JsRequest>();

        let responses = res_channel.clone();
        let h = handler.clone();
        spawn_local(async move {
            let res = h.handle_request(req).await.unwrap_or_else(|err| err);
            let status = res.status();
            let res = to_js_response(res).await;
            if !status.is_success() {
                log::warn!("{:?}", res);
            }
            responses.post_message(&res).expect("response");
        });
    }) as Box<dyn Fn(MessageEvent)>);
    req_channel.set_onmessage(Some(on_msg.as_ref().unchecked_ref()));
    on_msg.forget();

    Ok(())
}

fn load_service_worker(url: &str) -> Result<(), JsValue> {
    let _ = window()
        .ok_or("no window")?
        .navigator()
        .service_worker()
        .register(url);
    Ok(())
}

async fn to_js_response(mut response: Response) -> JsValue {
    let res = Object::new();
    Reflect::set(
        &res,
        &JsValue::from("status"),
        &JsValue::from(response.status() as u16),
    )
    .unwrap();
    let headers = response
        .iter()
        .map(|(name, val)| Array::of2(&JsValue::from(name.as_str()), &JsValue::from(val.as_str())))
        .collect::<Array>();
    Reflect::set(&res, &JsValue::from("headers"), &JsValue::from(&headers)).unwrap();
    let body = response.body_bytes().await.unwrap_or(vec![]);
    let body = Uint8Array::from(body.as_slice()).buffer();
    Reflect::set(&res, &JsValue::from("body"), &JsValue::from(body)).unwrap();
    res.into()
}

#[cfg(feature = "console_log")]
fn init_log() {
    use log::Level;
    console_log::init_with_level(Level::Debug).expect("error initializing log");
}
#[cfg(not(feature = "console_log"))]
fn init_log() {}
