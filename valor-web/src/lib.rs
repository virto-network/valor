#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use valor::{Handler, Request, Url};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn run() -> Result<(), JsValue> {
    init_log();
    let h = Handler::new();
    let req = Request::get(Url::parse("http://val.app").map_err(|_| "parse error")?);

    h.handle_request(req)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(())
}

#[cfg(feature = "console_log")]
fn init_log() {
    use log::Level;
    console_log::init_with_level(Level::Debug).expect("error initializing log");
}
#[cfg(not(feature = "console_log"))]
fn init_log() {}
