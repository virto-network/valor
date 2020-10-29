use valor::{Handler, Request, Url};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub async fn run() {
    log("Hola!");
    let h = Handler::new();
    h.handle_request(Request::get(Url::parse("/").unwrap()))
        .await
        .map_err(|e| log(&e.to_string()))
        .unwrap();
}
