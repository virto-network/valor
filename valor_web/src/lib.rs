//! Valor web
use loader::Loader;
use std::rc::Rc;
use std::sync::Arc;
use valor::{
    web::{into_js_response, into_request},
    Handler,
};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, BroadcastChannel, MessageEvent, RequestInit};

mod loader;

#[wasm_bindgen]
extern "C" {
    type TransferedRequest;
    #[wasm_bindgen(method, getter)]
    fn url(this: &TransferedRequest) -> String;
    #[wasm_bindgen(method, getter)]
    fn init(this: &TransferedRequest) -> RequestInit;
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
        let req = req.unchecked_into::<TransferedRequest>();
        let req = web_sys::Request::new_with_str_and_init(&req.url(), &req.init()).unwrap();

        let responses = res_channel.clone();
        let h = handler.clone();
        spawn_local(async move {
            let res = h
                .handle_request(into_request(req).await)
                .await
                .unwrap_or_else(|err| err);
            let status = res.status();
            let res = into_js_response(res).await;
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

#[cfg(feature = "console_log")]
fn init_log() {
    use log::Level;
    console_log::init_with_level(Level::Debug).expect("error initializing log");
}
#[cfg(not(feature = "console_log"))]
fn init_log() {}
