use async_std::io;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub async fn run() {
    log("Hola!");
    valor::handle_request(io::empty())
        .await
        .map_err(|e| log(&e.to_string()))
        .unwrap();
}
