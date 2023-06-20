use valor_runtime::{Runtime, Wasm};

fn main() {
    let rt = Runtime::with_defaults();
    let app = rt.load(include_bytes!("hello_service.wasm")).unwrap();
    rt.run(&app).unwrap();
}
