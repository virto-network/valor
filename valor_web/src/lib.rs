//! Valor web
use loader::Loader;
use std::rc::Rc;
use valor::{web::into_request, Vlugin};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, BroadcastChannel, MessageEvent, RequestInit};

mod loader;

#[wasm_bindgen]
extern "C" {
    type TransferredRequest;
    #[wasm_bindgen(method, getter)]
    fn url(this: &TransferredRequest) -> String;
    #[wasm_bindgen(method, getter)]
    fn init(this: &TransferredRequest) -> RequestInit;
}

/// Run
#[wasm_bindgen(start)]
pub async fn run() -> Result<(), JsValue> {
    init_log();
    load_service_worker("sw.js")?;

    let handler = Vlugin::new(Loader);

    let req_channel = BroadcastChannel::new("req_channel")?;
    let res_channel = Rc::new(BroadcastChannel::new("res_channel")?);

    // handle incoming requests from service worker
    // we get a JS Object with request data on the first BroadcastChannel
    // and must send response data through the sencond one in a timely
    // manner or the service worker will respond with a timeout
    let on_msg = Closure::wrap(Box::new(move |e: MessageEvent| {
        let req = e.data();
        let req = req.unchecked_ref::<TransferredRequest>();
        let req = web_sys::Request::new_with_str_and_init(&req.url(), &req.init())
            .expect("valid request");
        log::debug!("Got req: {:?}", req);

        let responses = res_channel.clone();
        let h = handler.clone();
        spawn_local(async move {
            let res = h
                .handle_request(into_request(req).await)
                .await
                .unwrap_or_else(|err| err);
            let status = res.status();
            if !status.is_success() {
                log::warn!("{:?}", res);
            }
            let res = transferable_response(res).await;
            log::debug!("posting res: {:?}", res);
            responses.post_message(&res).expect("response");
        });
    }) as Box<dyn Fn(MessageEvent)>);
    req_channel.set_onmessage(Some(on_msg.as_ref().unchecked_ref()));
    on_msg.forget();

    Ok(())
}

async fn transferable_response(mut res: valor::Response) -> JsValue {
    let body = res.body_bytes().await.unwrap_or_default();
    let body = js_sys::Uint8Array::from(body.as_slice()).buffer();
    let headers = js_sys::Object::new();
    for (name, value) in res.iter() {
        js_sys::Reflect::set(
            &headers,
            &JsValue::from(name.as_str()),
            &JsValue::from(value.as_str()),
        )
        .unwrap();
    }
    let status = res.status() as u16;
    let init = js_sys::Object::new();
    js_sys::Reflect::set(&init, &JsValue::from("status"), &status.into()).unwrap();
    js_sys::Reflect::set(&init, &JsValue::from("headers"), &headers.into()).unwrap();
    let res = js_sys::Object::new();
    js_sys::Reflect::set(&res, &JsValue::from("body"), &body.into()).unwrap();
    js_sys::Reflect::set(&res, &JsValue::from("init"), &init.into()).unwrap();
    res.into()
}

fn load_service_worker(url: &str) -> Result<(), JsValue> {
    let _ = window()
        .ok_or("no window")?
        .navigator()
        .service_worker()
        .register(url);
    Ok(())
}

#[cfg(feature = "console_log")]
fn init_log() {
    use log::Level;
    console_log::init_with_level(Level::Debug).expect("error initializing log");
}
#[cfg(not(feature = "console_log"))]
fn init_log() {}
