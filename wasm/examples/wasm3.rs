use std::error::Error;
use wasm::{Module, Runtime, Wasm};

fn main() {
    let runtime = Runtime::with_defaults();
    let module = runtime.load(include_bytes!("hello_service.wasm")).unwrap();
    module.start();
}
